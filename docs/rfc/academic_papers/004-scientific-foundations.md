# Academic Foundations & Research Lineage of KazeLab SynapseFlow

This document formalizes the peer-reviewed computer science literature and theoretical theorems underpinning the SynapseFlow platform.

---

## 1. PagedAttention: Virtual Memory for LLM Transformer KV Cache
- **Authors**: Woosuk Kwon et al. (UC Berkeley, SOSP 2023)
- **Problem**: KV cache in large context LLMs creates severe memory fragmentation ($>60-80\%$), drastically limiting agent concurrency.
- **Implementation in KazeLab**: [`crates/synapse-core/src/kv_paged/mod.rs`](file:///c:/Users/qt265/Downloads/CLAUDE/kazelab-web/crates/synapse-core/src/kv_paged/mod.rs).
- **Core Mechanism**: Non-contiguous physical page allocation for token blocks (`PAGE_BLOCK_SIZE = 16`) with atomic reference counting for instant prefix sharing between swarm subagents.

---

## 2. Modeling Software Vulnerabilities via Code Property Graphs (CPG)
- **Authors**: Fabian Yamaguchi, Nico Golde, Daniel Arp, Konrad Rieck (IEEE S&P 2014)
- **Problem**: Pure AST lacks control flow context; pure CFG lacks syntax structure.
- **Implementation in KazeLab**: [`crates/synapse-ast/src/cpg/mod.rs`](file:///c:/Users/qt265/Downloads/CLAUDE/kazelab-web/crates/synapse-ast/src/cpg/mod.rs).
- **Core Mechanism**: Unification of AST + CFG + PDG into a joint graph allowing single-pass graph queries for complex tainted data flows.

---

## 3. Automated STRIPS / PDDL Multi-Agent Planning
- **Authors**: Richard Fikes, Nils Nilsson (Artificial Intelligence 1971), Malik Ghallab et al. (2004)
- **Problem**: Pure LLM reasoning easily hallucinates circular reasoning loops or impossible state transitions.
- **Implementation in KazeLab**: [`crates/synapse-core/src/pddl/mod.rs`](file:///c:/Users/qt265/Downloads/CLAUDE/kazelab-web/crates/synapse-core/src/pddl/mod.rs) & [`backend/pddl_planner/planner.py`](file:///c:/Users/qt265/Downloads/CLAUDE/kazelab-web/backend/pddl_planner/planner.py).
- **Core Mechanism**: Grounded predicate forward state-space exploration ensuring provable mathematical reachability before invoking token-heavy foundation models.

---

## 4. DART: Directed Automated Random Testing (Concolic Execution)
- **Authors**: Patrice Godefroid, Nils Klarlund, Koushik Sen (PLDI 2005)
- **Problem**: Random test fuzzing cannot hit deep conditional branches guarded by complex integer equations.
- **Implementation in KazeLab**: [`crates/synapse-verifier/src/symbolic_symexec/mod.rs`](file:///c:/Users/qt265/Downloads/CLAUDE/kazelab-web/crates/synapse-verifier/src/symbolic_symexec/mod.rs).
- **Core Mechanism**: Execution trace gathering concrete inputs alongside symbolic path conditions, negating boundary branch conditions via SMT solvers to generate directed test vectors.
