# RFC 017: Inter-Procedural Taint Tracking & Automated CVE Vulnerability Eradication

- **Status**: Accepted
- **Author**: KazeLab Systems Architecture Team (Tú & ENI)
- **Target Subsystem**: `synapse-ast::interproc`, `synapse-verifier::cve`
- **Related Specifications**: RFC 004, CWE Top 25 (MITRE 2024), RFC 012

---

## 1. Abstract

Modern enterprise codebases suffer persistently from tainted input propagation leading to critical security vulnerabilities: Command Injection (CWE-78), SQL Injection (CWE-89), Cross-Site Scripting (CWE-79), and Path Traversal (CWE-22). Manual code review and superficial AST linters fail when tainted variables traverse across module boundaries, asynchronous task spawns, and dynamic traits.

RFC 017 specifies SynapseFlow's **Whole-Program Inter-Procedural Taint Analysis Engine**:
1. **Summary-Based Inter-Procedural Dataflow**: Computes summary transfer functions $f_{\text{summary}}: \mathcal{P}(\text{Sources}) \to \mathcal{P}(\text{Sinks})$ per function, enabling linear-time compositional analysis across massive projects.
2. **Path-Sensitive Sanitizer Recognition**: Validates that every path between source and sink traverses an approved cryptographically secure sanitizer function.
3. **Automated Remediating Synthesis**: When an un-sanitized sink is discovered, SynapseFlow automatically generates and splices parameterized sanitization wrappers into the AST before compiler emission.

---

## 2. Mathematical Transfer Function Formulation

Let $\mathbb{T}$ be the lattice of taint sets. The transfer function for basic block $B$ is defined as:

$$\text{OUT}[B] = \text{GEN}[B] \cup (\text{IN}[B] \setminus \text{KILL}[B])$$

Where:
- $\text{GEN}[B]$ denotes new taint sources introduced in block $B$ (e.g. network read, user input).
- $\text{KILL}[B]$ denotes sanitized expressions or bounded range checks.
- $\text{IN}[B] = \bigcup_{P \in \text{Pred}(B)} \text{OUT}[P]$.

A candidate implementation is flagged with a **Security Invariant Violation** if:
$$\exists s \in \text{Sinks}(B) \quad \text{such that} \quad s \cap \text{IN}[B] \ne \emptyset$$
