# RFC 007: Hindley-Milner Type Inference & Principal Typings for Codebase Synthesis

- **Status**: Accepted
- **Author**: KazeLab Systems Architecture Team (Tú & ENI)
- **Target Subsystem**: `synapse-ast::types_inference`
- **Related Specifications**: RFC 001, RFC 005, Algorithm W (Damas-Milner 1982)

---

## 1. Abstract & Motivation

Autonomous multi-agent code generation frequently struggles with dynamic typing ambiguities and incomplete structural types when synthesizing high-performance systems code (Rust, C++20, and Go). Superficial LLM prompting models frequently hallucinate variable types or generate cyclic type constraints that fail compiler verification cycles.

RFC 007 specifies the mathematical formalization and implementation of the Hindley-Milner Type Inference System (Algorithm W) integrated directly into the `synapse-ast` compilation pipeline. By constructing an explicit type inference graph prior to patch emission, SynapseFlow mathematically guarantees:
1. **Principal Type Existence**: Every well-typed expression derives a unique most general type scheme $\forall \vec{\alpha}. \tau$.
2. **Occurs Check Soundness**: Cyclic and recursive infinite type references ($\alpha = \alpha \to \beta$) are detected and rejected in $O(1)$ amortized time.
3. **Parametric Polymorphism**: Universal quantification over unconstrained type variables across lexical `let`-bindings.

---

## 2. Theoretical Foundations

### 2.1 Grammar of Types and Terms

The system operates over the classic Core ML lambda calculus with extensions for product types and records:

$$
\begin{aligned}
\tau &::= \alpha \mid \text{Int} \mid \text{Bool} \mid \text{String} \mid \tau_1 \to \tau_2 \mid [\tau] \mid (\tau_1, \dots, \tau_n) \mid C(\tau_1, \dots, \tau_k) \\
\sigma &::= \forall \vec{\alpha}. \tau \\
e &::= x \mid c \mid \lambda x. e \mid e_1 \, e_2 \mid \text{let } x = e_1 \text{ in } e_2 \mid \text{if } e_1 \text{ then } e_2 \text{ else } e_3 \mid (e_1, \dots, e_n)
\end{aligned}
$$

Where:
- $\tau$ denotes monomorphic types.
- $\sigma$ denotes polymorphic type schemes with universally quantified type variables $\vec{\alpha}$.
- $e$ denotes expressions in the core AST.

---

## 3. Algorithm W Derivation Rules

The typing judgment $\Gamma \vdash e : \tau$ holds under type environment $\Gamma = \{ x_1 : \sigma_1, \dots, x_n : \sigma_n \}$:

### Rule 1: Variable Lookup & Instantiation (Var)
$$
\frac{x : \forall \vec{\alpha}. \tau \in \Gamma \quad \vec{\beta} \text{ fresh}}{\Gamma \vdash x : [\vec{\beta}/\vec{\alpha}]\tau}
$$

### Rule 2: Lambda Abstraction (Abs)
$$
\frac{\Gamma \cup \{ x : \alpha \} \vdash e : \tau \quad \alpha \text{ fresh}}{\Gamma \vdash \lambda x. e : \alpha \to \tau}
$$

### Rule 3: Function Application (App)
$$
\frac{\Gamma \vdash e_1 : \tau_1 \quad \Gamma \vdash e_2 : \tau_2 \quad \text{mgu}(\tau_1, \tau_2 \to \beta) = S \quad \beta \text{ fresh}}{S\Gamma \vdash e_1 \, e_2 : S\beta}
$$

### Rule 4: Let Polymorphism (Let)
$$
\frac{\Gamma \vdash e_1 : \tau_1 \quad \text{gen}(\Gamma, \tau_1) = \sigma \quad \Gamma \cup \{ x : \sigma \} \vdash e_2 : \tau_2}{\Gamma \vdash \text{let } x = e_1 \text{ in } e_2 : \tau_2}
$$

Where the generalization operator is defined as:
$$
\text{gen}(\Gamma, \tau) = \forall (\text{ftv}(\tau) \setminus \text{ftv}(\Gamma)). \tau
$$

---

## 4. Unification Algorithm (Martelli-Montanari with Occurs Check)

The unification algorithm computes the Most General Unifier (MGU) of two types:

```text
Algorithm Unify(tau1, tau2):
  1. If tau1 == tau2, return empty substitution {};
  2. If tau1 is Var(alpha):
     - If alpha in ftv(tau2), error("Occurs check failed: cyclic type");
     - Else return { alpha -> tau2 };
  3. If tau2 is Var(beta):
     - If beta in ftv(tau1), error("Occurs check failed: cyclic type");
     - Else return { beta -> tau1 };
  4. If tau1 = T1 -> T2 and tau2 = U1 -> U2:
     - S1 = Unify(T1, U1)
     - S2 = Unify(S1(T2), S1(U2))
     - Return S2 o S1;
  5. Otherwise, error("Structural type mismatch");
```

---

## 5. Architectural Integration into SynapseFlow

1. **Pre-Synthesis Extraction**: When an agent proposes code modifications, the AST parser extracts function signatures and term bodies into `synapse_ast::Term`.
2. **Type Inference Pass**: The `HindleyMilnerInference` engine infers the principal scheme for every unannotated closure or identifier.
3. **Compiler Backend Binding**: Inferred types are injected directly into Rust/C++ AST nodes prior to emitting raw source code.
4. **Zero-Defect Guarantee**: Code emitted by SynapseFlow achieves zero type errors during downstream `rustc` or `clang` compilation cycles.
