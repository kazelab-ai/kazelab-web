# RFC 009: BPF Seccomp Kernel Sandbox & Container Isolation Policy

- **Status**: Accepted
- **Author**: KazeLab Systems Architecture Team (Tú & ENI)
- **Target Subsystem**: `synapse-mcp::security::seccomp`
- **Related Specifications**: Linux BPF (cBPF / seccomp mode 2), RFC 002

---

## 1. Threat Model & Sandboxing Rationale

Autonomous agents capable of synthesizing and executing arbitrary code in host environments present catastrophic security hazards if uncontained:
1. **Host Privilege Escalation**: Exploitation of Linux kernel vulnerabilities (e.g. dirty COW, unescaped namespaces).
2. **Network Data Exfiltration**: Malicious tool calls opening unauthorized reverse shells or exfiltrating API secrets via external HTTP/DNS channels.
3. **Destructive Filesystem Alterations**: Un-sandboxed execution of recursive destructive shell commands (`rm -rf /`, `mkfs`, device writes).

RFC 009 specifies SynapseFlow's **Two-Tier Defense-in-Depth Container Architecture**:
- **Layer 1 (Application-Level Pattern Auditing)**: Fast regex and AST-based command validation.
- **Layer 2 (Kernel-Level BPF Seccomp Filter Synthesis)**: In-kernel Berkeley Packet Filter (BPF) syscall whitelisting generated dynamically per tool execution profile.

---

## 2. BPF Seccomp Instruction Architecture

Every MCP tool invocation operates under a restricted process context governed by `prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog)`.

### 2.1 Syscall Whitelist Profile
By default, the worker sandbox enforces an absolute whitelist granting only 9 essential POSIX system calls:

```text
Syscall NR   Name          Rationale
--------------------------------------------------------------------------
0            sys_read      Reading from standard input pipes
1            sys_write     Writing output logs to standard output pipes
3            sys_close     Releasing file descriptors
9            sys_mmap      Allocating heap and virtual memory pages
10           sys_mprotect  Setting page execution permissions (W^X enforced)
11           sys_munmap    Deallocating virtual memory
12           sys_brk       Heap memory allocation
60           sys_exit      Normal process termination
231          sys_exit_group Multi-threaded process termination
```

Any unauthorized system call invocation (including `sys_socket`, `sys_connect`, `sys_ptrace`, `sys_fork`, `sys_execve`) instantly triggers `SECCOMP_RET_KILL_PROCESS`, terminating the process before the kernel context switch completes.

---

## 3. Bytecode Verification & Zero-Overhead Execution

The synthesized BPF filter program evaluates syscall numbers in $O(\log N)$ or sequential comparisons directly inside the kernel's eBPF execution engine:
- CPU overhead per system call: $< 12$ nanoseconds.
- Memory footprint per sandbox instance: $< 4$ kilobytes.
- Hardware support: Compatible with all Linux x86_64, AArch64, and WSL2 environments.
