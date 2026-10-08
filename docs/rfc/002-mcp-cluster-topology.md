# RFC 002: Model Context Protocol (MCP 1.1) Cluster Topology

- **Author**: Tú (KazeLAB) & KazeLab Systems Architecture Team
- **Status**: Standard / Accepted
- **Created**: 2024-11-15
- **Updated**: 2026-10-08

---

## 1. Abstract
This RFC establishes the topology, security envelope, and communication protocols for KazeLab's native Model Context Protocol (MCP 1.1) cluster.

---

## 2. Cluster Topology

```
+-----------------------------------------------------------+
|              SynapseFlow Core Orchestrator                |
+-----------------------------------------------------------+
       |                     |                     |
       v                     v                     v
+---------------+     +---------------+     +---------------+
| mcp-ast       |     | mcp-sandbox   |     | mcp-security  |
| Analyzer Node |     | Runner Node   |     | Auditor Node  |
| (Tree-sitter) |     | (Cargo/Clang) |     | (Symbolic)    |
+---------------+     +---------------+     +---------------+
```

---

## 3. Tool Invocation Invariants
1. **Isolated Namespaces**: Tool execution occurs in cgroup-isolated ephemeral sandboxes with network egress filtering.
2. **Deterministic Schemas**: All tool inputs and outputs must strictly validate against JSON Schema Draft 2020-12.
3. **Sub-5ms IPC**: Tool calls within the local cluster utilize shared-memory ring buffers or domain sockets.
