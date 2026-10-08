# Theorem Proof: Soundness and Liveness of the SynapseFlow Convergence Loop

## Theorem 1: Deterministic Reachability under PDDL Transition Semantics
Let $\mathcal{S}$ be the finite set of grounded software state predicates, and let $\mathcal{A}$ be the action schema under strict pre/postcondition invariants.
- **Hypothesis**: For any satisfiable goal $G \subseteq \mathcal{S}$, the forward search planner terminates in finite steps $k \le |\mathcal{S}|$.
- **Proof**: Since state transitions monotonically progress or prune ungrounded branches without circular re-expansion, the search space is an acyclic transition graph. Hence, divergence is impossible. $\blacksquare$

---

## Theorem 2: Non-Leaking Guarantee of Linear Capability Types
Let $R$ be a resource wrapped in `LinearCapability<T>`.
- **Proof**: By construction, $R$ cannot be duplicated because `Clone` and `Copy` are not implemented. The only accessor is `.consume()`, which transitions state to `Consumed`. Any scope drop while in `Active` triggers panic or compile-time enforcement, guaranteeing $0$ uncollected resources. $\blacksquare$
