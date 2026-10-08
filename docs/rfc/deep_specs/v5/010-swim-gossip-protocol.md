# RFC 010: SWIM Failure Detection & Gossip-Based Mesh Scalability

- **Status**: Accepted
- **Author**: KazeLab Systems Architecture Team (Tú & ENI)
- **Target Subsystem**: `synapse-core::actor::cluster`
- **Related Specifications**: SWIM Protocol (Das et al., IEEE DSN 2002), RFC 003

---

## 1. Abstract

Large multi-agent swarms operating across distributed nodes require fault tolerance and dynamic membership tracking without central bottlenecks. Traditional heartbeat mechanisms with $O(N^2)$ message complexity overload network infrastructure as swarms expand beyond dozens of agents.

RFC 010 specifies the integration of the **Structured Weakly-Consistent Infection-Style Process Group Membership Protocol (SWIM)** into SynapseFlow's core actor system. SWIM guarantees:
1. **$O(1)$ Message Overhead**: Constant network overhead per node per gossip period $T$.
2. **Deterministic Bound on False Positives**: Two-phase ping-ack and indirect probing (`PingReq`) virtually eradicate false positives caused by sporadic network jitter.
3. **Suspicion Mechanism**: Nodes suspected of failure are allowed an incubation period $\tau$ to refute false failure detections before permanent eviction.

---

## 2. Protocol State Machine

Each cluster member maintains a local membership table with states:
$$\text{State} \in \{ \text{Alive}, \text{Suspect}, \text{Dead}, \text{Left} \}$$

### 2.1 Direct Probing
At every protocol period $T$:
1. A node $p_i$ randomly selects a peer $p_j$ from its member list and sends a `Ping(seq)`.
2. If $p_i$ receives `Ack(seq)` within timeout $t_{\text{ack}}$, $p_j$ is marked `Alive`.

### 2.2 Indirect Probing (`PingReq`)
If no acknowledgment arrives within $t_{\text{ack}}$:
1. $p_i$ selects $k$ random auxiliary peers $\{ q_1, \dots, q_k \}$.
2. $p_i$ sends `PingReq(target: p_j)` to each auxiliary peer.
3. Each $q_m$ immediately pings $p_j$ on behalf of $p_i$ and forwards any received acknowledgment back to $p_i$.
4. If any forwarded acknowledgment arrives, $p_j$ remains `Alive`.

### 2.3 Suspicion & Incarnation Counter
If all indirect pings fail:
1. $p_j$ is transitioned to `Suspect` status with incarnation number $\lambda$.
2. `Suspect(p_j, \lambda)` is disseminated across the cluster via gossip piggybacking.
3. If $p_j$ is alive, it refutes the suspicion by broadcasting `Alive(p_j, \lambda + 1)`.
4. If no refutation is received within incubation time $\tau$, $p_j$ is declared `Dead` and evicted.
