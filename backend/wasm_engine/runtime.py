"""
Sandboxed WebAssembly (Wasm) Runtime Harness.
Executes un-trusted micro-extensions with strict memory limits and execution fuel limits.
"""

from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Any
import struct


class WasmTrapException(Exception):
    """Raised when a WebAssembly execution violates memory bounds or traps."""
    pass


class WasmFuelExhaustedException(Exception):
    """Raised when WebAssembly execution exceeds allocated CPU fuel."""
    pass


@dataclass
class WasmMemory:
    """Simulated 64KB paged WebAssembly Linear Memory."""
    pages: int = 1
    max_pages: int = 16
    page_size: int = 65536
    buffer: bytearray = field(init=False)

    def __post_init__(self):
        self.buffer = bytearray(self.pages * self.page_size)

    def grow(self, additional_pages: int) -> int:
        old_pages = self.pages
        if self.pages + additional_pages > self.max_pages:
            return -1
        self.pages += additional_pages
        self.buffer.extend(bytearray(additional_pages * self.page_size))
        return old_pages

    def load_i32(self, offset: int) -> int:
        if offset + 4 > len(self.buffer) or offset < 0:
            raise WasmTrapException(f"Out of bounds memory read at offset {offset}")
        return struct.unpack_from("<i", self.buffer, offset)[0]

    def store_i32(self, offset: int, value: int) -> None:
        if offset + 4 > len(self.buffer) or offset < 0:
            raise WasmTrapException(f"Out of bounds memory write at offset {offset}")
        struct.pack_into("<i", self.buffer, offset, value)

    def load_bytes(self, offset: int, length: int) -> bytes:
        if offset + length > len(self.buffer) or offset < 0:
            raise WasmTrapException(f"Out of bounds memory slice load at offset {offset}, len {length}")
        return bytes(self.buffer[offset:offset + length])

    def store_bytes(self, offset: int, data: bytes) -> None:
        if offset + len(data) > len(self.buffer) or offset < 0:
            raise WasmTrapException(f"Out of bounds memory slice store at offset {offset}, len {len(data)}")
        self.buffer[offset:offset + len(data)] = data


@dataclass
class WasmStackFrame:
    func_name: str
    locals: List[int] = field(default_factory=list)
    return_ip: int = 0


class WasmSandboxedEngine:
    """
    Interpreted Stack-based WebAssembly virtual execution engine.
    Guarantees isolation and fuel consumption bounds.
    """

    def __init__(self, initial_fuel: int = 100_000):
        self.memory = WasmMemory(pages=2, max_pages=16)
        self.operand_stack: List[int] = []
        self.call_stack: List[WasmStackFrame] = []
        self.fuel = initial_fuel
        self.exported_functions: Dict[str, Any] = {}
        self.host_imports: Dict[str, Any] = {}

    def register_host_import(self, module: str, name: str, func: Any) -> None:
        self.host_imports[f"{module}::{name}"] = func

    def consume_fuel(self, amount: int = 1) -> None:
        if self.fuel < amount:
            raise WasmFuelExhaustedException("WASM execution fuel exhausted")
        self.fuel -= amount

    def push(self, val: int) -> None:
        self.operand_stack.append(val)

    def pop(self) -> int:
        if not self.operand_stack:
            raise WasmTrapException("Operand stack underflow")
        return self.operand_stack.pop()

    def execute_op(self, opcode: str, *args) -> None:
        self.consume_fuel(1)
        if opcode == "i32.const":
            self.push(args[0])
        elif opcode == "i32.add":
            b, a = self.pop(), self.pop()
            self.push((a + b) & 0xFFFFFFFF)
        elif opcode == "i32.sub":
            b, a = self.pop(), self.pop()
            self.push((a - b) & 0xFFFFFFFF)
        elif opcode == "i32.mul":
            b, a = self.pop(), self.pop()
            self.push((a * b) & 0xFFFFFFFF)
        elif opcode == "i32.load":
            offset = self.pop()
            val = self.memory.load_i32(offset)
            self.push(val)
        elif opcode == "i32.store":
            val = self.pop()
            offset = self.pop()
            self.memory.store_i32(offset, val)
        else:
            raise WasmTrapException(f"Unsupported Wasm opcode: {opcode}")

    def run_bytecode_program(self, instructions: List[Tuple[str, List[Any]]]) -> int:
        """Runs a sequence of simulated WASM IR instructions and returns the top stack value."""
        for instr in instructions:
            opcode, params = instr[0], instr[1]
            self.execute_op(opcode, *params)
        return self.pop() if self.operand_stack else 0
