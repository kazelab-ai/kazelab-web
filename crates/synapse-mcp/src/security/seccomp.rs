//! Linux Berkeley Packet Filter (BPF) Seccomp Filter Synthesis Engine.
//! Generates strict system call filtering bytecode for sandboxed tool execution.

use serde::{Deserialize, Serialize};

pub const BPF_LD: u16 = 0x00;
pub const BPF_W: u16 = 0x00;
pub const BPF_ABS: u16 = 0x20;
pub const BPF_JMP: u16 = 0x05;
pub const BPF_JEQ: u16 = 0x10;
pub const BPF_K: u16 = 0x00;
pub const BPF_RET: u16 = 0x06;

pub const SECCOMP_RET_KILL_PROCESS: u32 = 0x80000000;
pub const SECCOMP_RET_TRAP: u32 = 0x00030000;
pub const SECCOMP_RET_ERRNO: u32 = 0x00050000;
pub const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;

pub const SECCOMP_DATA_NR_OFFSET: u32 = 0;
pub const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
pub const AUDIT_ARCH_X86_64: u32 = 0xc000003e;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BpfInstruction {
    pub code: u16,
    pub jt: u8,
    pub jf: u8,
    pub k: u32,
}

impl BpfInstruction {
    pub fn stmt(code: u16, k: u32) -> Self {
        Self { code, jt: 0, jf: 0, k }
    }

    pub fn jump(code: u16, k: u32, jt: u8, jf: u8) -> Self {
        Self { code, jt, jf, k }
    }
}

pub struct SeccompProfileBuilder {
    allowed_syscalls: Vec<u32>,
    default_action: u32,
    arch: u32,
}

impl SeccompProfileBuilder {
    pub fn new() -> Self {
        Self {
            allowed_syscalls: Vec::new(),
            default_action: SECCOMP_RET_KILL_PROCESS,
            arch: AUDIT_ARCH_X86_64,
        }
    }

    pub fn allow_syscall(mut self, nr: u32) -> Self {
        if !self.allowed_syscalls.contains(&nr) {
            self.allowed_syscalls.push(nr);
        }
        self
    }

    pub fn set_default_action(mut self, action: u32) -> Self {
        self.default_action = action;
        self
    }

    pub fn compile_bpf_program(&self) -> Vec<BpfInstruction> {
        let mut filter = Vec::new();

        // 1. Verify Architecture (validate syscall calling convention)
        filter.push(BpfInstruction::stmt(BPF_LD | BPF_W | BPF_ABS, SECCOMP_DATA_ARCH_OFFSET));
        filter.push(BpfInstruction::jump(BPF_JMP | BPF_JEQ | BPF_K, self.arch, 1, 0));
        filter.push(BpfInstruction::stmt(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS));

        // 2. Load Syscall Number
        filter.push(BpfInstruction::stmt(BPF_LD | BPF_W | BPF_ABS, SECCOMP_DATA_NR_OFFSET));

        // 3. Check allowed syscalls
        let count = self.allowed_syscalls.len();
        for (i, &nr) in self.allowed_syscalls.iter().enumerate() {
            let remaining = (count - 1 - i) as u8;
            filter.push(BpfInstruction::jump(
                BPF_JMP | BPF_JEQ | BPF_K,
                nr,
                remaining + 1, // Jump over remaining checks directly to ALLOW
                0,             // Check next instruction
            ));
        }

        // 4. Default return (e.g. KILL or ERRNO)
        filter.push(BpfInstruction::stmt(BPF_RET | BPF_K, self.default_action));

        // 5. Allow return
        filter.push(BpfInstruction::stmt(BPF_RET | BPF_K, SECCOMP_RET_ALLOW));

        filter
    }

    pub fn standard_sandbox_policy() -> Self {
        let mut builder = Self::new().set_default_action(SECCOMP_RET_ERRNO | 1); // EPERM
        // Minimal standard POSIX syscalls for pure computation & pipe IPC
        let syscalls = [
            0,   // read
            1,   // write
            3,   // close
            9,   // mmap
            10,  // mprotect
            11,  // munmap
            12,  // brk
            60,  // exit
            231, // exit_group
        ];
        for nr in syscalls {
            builder = builder.allow_syscall(nr);
        }
        builder
    }
}
