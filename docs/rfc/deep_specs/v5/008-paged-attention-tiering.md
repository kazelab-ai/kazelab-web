# RFC 008: Tiered KV-Cache Architecture and NVMe Virtual Page Eviction

- **Status**: Accepted
- **Author**: KazeLab Systems Architecture Team (Tú & ENI)
- **Target Subsystem**: `synapse-core::kv_paged::tiering`
- **Related Specifications**: RFC 001, RFC 003, vLLM PagedAttention (Kwon et al., SOSP 2023)

---

## 1. Abstract

Large Language Model swarms operating over extensive software repositories encounter severe memory bottlenecks due to Key-Value (KV) cache expansion. For long-context reasoning with Claude 3.5 Sonnet across hundreds of inter-procedural AST call graphs, storing uncompressed KV blocks exclusively in High Bandwidth Memory (HBM) results in catastrophic out-of-memory (OOM) aborts or exorbitant operational costs.

RFC 008 defines the architecture for SynapseFlow's **3-Tier Hierarchical KV-Cache Engine**:
1. **Tier 1 (GPU HBM / On-Die SRAM)**: Sub-microsecond access latency for active generation tokens.
2. **Tier 2 (Host DDR5 DRAM)**: High-throughput memory buffer over PCIe Gen 5 x16 (64 GB/s theoretical bandwidth).
3. **Tier 3 (Host NVMe Direct I/O SSD)**: Zero-copy block-addressed disk storage providing virtually infinite context window capacity at negligible cost.

---

## 2. Memory Tier Invariants & Latency Targets

```
+-------------------------------------------------------------------------+
| Tier 1: GPU HBM (High Bandwidth Memory)                                 |
| Capacity: 16k - 64k Tokens  | Latency: < 50 ns  | Bandwidth: 2.0 TB/s   |
+-------------------------------------------------------------------------+
                                    |  (Async Page Migration)
                                    v
+-------------------------------------------------------------------------+
| Tier 2: Host System RAM (DDR5 Dual-Channel)                            |
| Capacity: 500k - 2M Tokens  | Latency: < 80 ns  | Bandwidth: 64 GB/s    |
+-------------------------------------------------------------------------+
                                    |  (Direct I/O Spilling)
                                    v
+-------------------------------------------------------------------------+
| Tier 3: Local NVMe SSD (PCIe Gen 4/5)                                   |
| Capacity: 10M+ Tokens       | Latency: < 15 us  | Bandwidth: 7.0 GB/s   |
+-------------------------------------------------------------------------+
```

---

## 3. Physical Block Virtualization & Prefix Sharing

Following vLLM PagedAttention principles, logical KV caches are partitioned into uniform physical blocks of $B = 16$ tokens:

$$\text{BlockSize} = 2 \times N_{\text{layers}} \times N_{\text{heads}} \times D_{\text{head}} \times B \times \text{sizeof(FP16)}$$

For standard 70B parameter models ($N_{\text{layers}} = 80, N_{\text{heads}} = 64, D_{\text{head}} = 128$), each block requires:
$$\text{BlockSize} = 2 \times 80 \times 64 \times 128 \times 16 \times 2 = 41,943,040 \text{ bytes} \approx 40 \text{ MB}$$

### 3.1 Structural Prefix Sharing
When multiple swarm agents (Cognitive Architect, Systems Coder, Verification Engine) evaluate identical repository context:
1. The common repository prompt prefix is mapped to identical physical block IDs.
2. Reference counts (`ref_count`) are incremented atomically via `AtomicUsize`.
3. Memory consumption scales with $O(1)$ for the shared codebase prefix rather than $O(N)$ per agent.

---

## 4. Eviction & Migration Policy (Frequency-Aware CLOCK Algorithm)

To minimize cache thrashing across memory tiers, SynapseFlow employs a multi-tiered CLOCK replacement policy:

$$
\text{PriorityScore}(b) = w_1 \cdot \text{AccessCount}(b) + w_2 \cdot e^{-\lambda \Delta t} + w_3 \cdot \text{DepthLevel}(b)
$$

Where:
- $\Delta t$ is the elapsed time since the block was last accessed.
- $\text{DepthLevel}(b)$ prioritizes prompt root blocks over leaf tokens.
- Blocks with $\text{PriorityScore} < \theta_{\text{demote}}$ are demoted sequentially:
  $$\text{HBM} \xrightarrow{\text{DMA}} \text{Host DRAM} \xrightarrow{\text{O\_DIRECT}} \text{NVMe SSD}$$
- When a prompt prefix is referenced again, blocks are pre-fetched asynchronously in the background.

---

## 5. Security & Isolation Invariants

1. **Cryptographic Scrambling**: Blocks evicted to NVMe SSD are encrypted at rest using AES-256-GCM with an ephemeral hardware key derived from the swarm session token.
2. **Zero Residual Remanence**: When a swarm session completes, all physical blocks across all three tiers are overwritten with zeros (`0x00`) before deallocation.
