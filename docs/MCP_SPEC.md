# Model Context Protocol (MCP) Cluster Specification

## Overview
KazeLab SynapseFlow natively communicates with tooling clusters using Anthropic's **Model Context Protocol (MCP) v1.1.0** specification.

---

## Registered Toolchain Nodes

### 1. `mcp-ast-analyzer`
- **Endpoint**: `mcp://ast.cluster.kazelab.xyz:9001`
- **Capabilities**:
  - `parse_ast`: Extract high-order syntax trees using tree-sitter.
  - `extract_symbols`: Build symbol dictionaries and interface contracts.
  - `build_dependency_dag`: Compute directed acyclic graphs of modules.

### 2. `mcp-sandbox-runner`
- **Endpoint**: `mcp://runner.cluster.kazelab.xyz:9002`
- **Capabilities**:
  - `cargo_check`: Rust borrow-checker and unit verification.
  - `clang_format`: C++20 formatting and compile verification.
  - `pytest_isolated`: Sandboxed Python test runner.

### 3. `mcp-git-engine`
- **Endpoint**: `mcp://git.cluster.kazelab.xyz:9003`
- **Capabilities**:
  - `synthesize_patch`: Generate atomic unified diff patches.
  - `commit_signed`: GPG-signed git commits.
  - `verify_pr`: Branch comparison and merge conflict resolution.

### 4. `mcp-security-auditor`
- **Endpoint**: `mcp://sec.cluster.kazelab.xyz:9004`
- **Capabilities**:
  - `scan_memory_safety`: Symbolic execution for buffer overruns and use-after-free.
  - `audit_cve`: Dependency vulnerability scanning against OSV database.
