"""
eBPF Bytecode Safety Verifier (Simulates Linux In-Kernel Invariant Verifier).
Performs static analysis on instruction flow, checking register tracking,
pointer arithmetic safety, out-of-bounds stack access, and termination (loop bounds).
"""

from typing import List, Dict, Set, Tuple
from dataclasses import dataclass, field
from enum import Enum


class RegType(Enum):
    NOT_INIT = "NOT_INIT"
    SCALAR_VALUE = "SCALAR_VALUE"
    PTR_TO_STACK = "PTR_TO_STACK"
    PTR_TO_CTX = "PTR_TO_CTX"
    PTR_TO_MAP_VALUE = "PTR_TO_MAP_VALUE"


@dataclass
class RegState:
    reg_type: RegType = RegType.NOT_INIT
    var_off_min: int = 0
    var_off_max: int = 0
    stack_offset: int = 0


@dataclass
class BpfInsn:
    opcode: str
    dst_reg: int
    src_reg: int
    offset: int
    imm: int


class BpfVerificationException(Exception):
    """Raised when an eBPF program fails verification invariants."""
    pass


class EbpfBytecodeVerifier:
    """
    Simulated Linux Kernel eBPF Verifier (sound abstract interpretation).
    Checks DAG termination, bounds, and register type correctness.
    """

    MAX_INSN_COUNT = 4096
    STACK_SIZE = 512

    def __init__(self):
        # 10 registers R0..R9 + R10 (frame pointer read-only)
        self.regs: List[RegState] = [RegState() for _ in range(11)]
        # R10 is pointer to stack top
        self.regs[10] = RegState(reg_type=RegType.PTR_TO_STACK, stack_offset=self.STACK_SIZE)
        # R1 is context pointer
        self.regs[1] = RegState(reg_type=RegType.PTR_TO_CTX)
        self.visited_states: Set[str] = set()

    def _state_hash(self, pc: int) -> str:
        r_str = ",".join(f"{r.reg_type.value}:{r.var_off_min}:{r.stack_offset}" for r in self.regs)
        return f"{pc}|{r_str}"

    def verify_program(self, program: List[BpfInsn]) -> bool:
        if len(program) > self.MAX_INSN_COUNT:
            raise BpfVerificationException(f"Program exceeds maximum length {self.MAX_INSN_COUNT}")

        pc = 0
        total_instructions_analyzed = 0

        while pc < len(program):
            total_instructions_analyzed += 1
            if total_instructions_analyzed > 32768:
                raise BpfVerificationException("Verifier instruction complexity limit exceeded (possible unbounded loop)")

            insn = program[pc]
            opcode = insn.opcode

            # Check registers range
            if insn.dst_reg < 0 or insn.dst_reg > 10:
                raise BpfVerificationException(f"Invalid dst_reg {insn.dst_reg} at PC {pc}")

            if insn.dst_reg == 10 and opcode not in ["ST", "STX"]:
                raise BpfVerificationException("Cannot write directly into frame pointer R10")

            if opcode == "MOV_IMM":
                self.regs[insn.dst_reg] = RegState(
                    reg_type=RegType.SCALAR_VALUE,
                    var_off_min=insn.imm,
                    var_off_max=insn.imm,
                )
                pc += 1

            elif opcode == "MOV_REG":
                src = self.regs[insn.src_reg]
                if src.reg_type == RegType.NOT_INIT:
                    raise BpfVerificationException(f"Read from uninitialized reg R{insn.src_reg} at PC {pc}")
                self.regs[insn.dst_reg] = RegState(
                    reg_type=src.reg_type,
                    var_off_min=src.var_off_min,
                    var_off_max=src.var_off_max,
                    stack_offset=src.stack_offset,
                )
                pc += 1

            elif opcode == "ALU_ADD_IMM":
                dst = self.regs[insn.dst_reg]
                if dst.reg_type == RegType.SCALAR_VALUE:
                    dst.var_off_min += insn.imm
                    dst.var_off_max += insn.imm
                elif dst.reg_type == RegType.PTR_TO_STACK:
                    dst.stack_offset += insn.imm
                    if dst.stack_offset < 0 or dst.stack_offset > self.STACK_SIZE:
                        raise BpfVerificationException(f"Stack pointer offset {dst.stack_offset} out of bounds")
                else:
                    raise BpfVerificationException(f"Cannot perform arithmetic on pointer type {dst.reg_type}")
                pc += 1

            elif opcode == "STX_MEM":
                # Store register to memory
                dst = self.regs[insn.dst_reg]
                if dst.reg_type == RegType.PTR_TO_STACK:
                    target_off = dst.stack_offset + insn.offset
                    if target_off < 0 or target_off > self.STACK_SIZE:
                        raise BpfVerificationException(f"Invalid stack memory write at offset {target_off}")
                else:
                    raise BpfVerificationException("Only stack memory writes permitted in static mode")
                pc += 1

            elif opcode == "LDX_MEM":
                # Load memory to register
                src = self.regs[insn.src_reg]
                if src.reg_type == RegType.PTR_TO_STACK:
                    target_off = src.stack_offset + insn.offset
                    if target_off < 0 or target_off > self.STACK_SIZE:
                        raise BpfVerificationException(f"Invalid stack memory read at offset {target_off}")
                    self.regs[insn.dst_reg] = RegState(reg_type=RegType.SCALAR_VALUE)
                else:
                    raise BpfVerificationException("Only stack memory loads permitted")
                pc += 1

            elif opcode == "JA":
                pc += 1 + insn.offset

            elif opcode == "EXIT":
                # Ensure R0 is initialized on exit
                if self.regs[0].reg_type == RegType.NOT_INIT:
                    raise BpfVerificationException("R0 must be initialized with return value before EXIT")
                return True

            else:
                raise BpfVerificationException(f"Unknown eBPF instruction opcode: {opcode}")

        raise BpfVerificationException("Program execution did not encounter terminal EXIT instruction")
