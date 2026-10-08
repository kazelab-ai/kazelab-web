//! Register-based Mid-Level Intermediate Representation (MIR) Abstract Machine Interpreter.
//! Used for concrete and symbolic stepping, memory bounds verification, and abstract interpretation.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirLiteral {
    Int(i64),
    Bool(bool),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirOp {
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    Lt,
    Gt,
}

#[derive(Debug, Clone)]
pub enum MirInst {
    LoadConst(Reg, MirLiteral),
    Copy(Reg, Reg),
    BinOp(Reg, MirOp, Reg, Reg),
    Alloc(Reg, usize),              // Allocates heap block of size N, puts handle in dst reg
    Store(Reg, usize, Reg),         // Store ptr_reg + offset <- val_reg
    Load(Reg, Reg, usize),          // Load dst_reg <- ptr_reg + offset
    Branch(Reg, usize, usize),      // If cond_reg goto target1 else target2
    Jump(usize),                    // Unconditional jump to block index
    Call(Reg, String, Vec<Reg>),    // dst_reg = call fn_name(args)
    Return(Option<Reg>),
}

#[derive(Debug, Clone)]
pub struct MirBasicBlock {
    pub id: usize,
    pub instructions: Vec<MirInst>,
}

#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<Reg>,
    pub blocks: HashMap<usize, MirBasicBlock>,
    pub entry_block: usize,
}

#[derive(Debug, Clone, Default)]
pub struct MirMachineState {
    pub registers: HashMap<Reg, MirLiteral>,
    pub memory: HashMap<u64, Vec<u8>>,
    pub next_alloc_ptr: u64,
    pub pc_block: usize,
    pub pc_inst: usize,
    pub halted: bool,
    pub return_value: Option<MirLiteral>,
}

impl MirMachineState {
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            memory: HashMap::new(),
            next_alloc_ptr: 0x1000,
            pc_block: 0,
            pc_inst: 0,
            halted: false,
            return_value: None,
        }
    }

    pub fn set_reg(&mut self, reg: Reg, val: MirLiteral) {
        self.registers.insert(reg, val);
    }

    pub fn get_reg(&self, reg: Reg) -> Option<&MirLiteral> {
        self.registers.get(&reg)
    }

    pub fn get_reg_int(&self, reg: Reg) -> Result<i64, String> {
        match self.registers.get(&reg) {
            Some(MirLiteral::Int(v)) => Ok(*v),
            Some(_) => Err(format!("Register {:?} holds non-integer value", reg)),
            None => Err(format!("Uninitialized register {:?}", reg)),
        }
    }

    pub fn get_reg_bool(&self, reg: Reg) -> Result<bool, String> {
        match self.registers.get(&reg) {
            Some(MirLiteral::Bool(v)) => Ok(*v),
            Some(_) => Err(format!("Register {:?} holds non-bool value", reg)),
            None => Err(format!("Uninitialized register {:?}", reg)),
        }
    }
}

pub struct MirInterpreter {
    pub function: MirFunction,
    pub state: MirMachineState,
}

impl MirInterpreter {
    pub fn new(function: MirFunction) -> Self {
        let entry = function.entry_block;
        let mut state = MirMachineState::new();
        state.pc_block = entry;
        Self { function, state }
    }

    pub fn set_args(&mut self, args: Vec<MirLiteral>) -> Result<(), String> {
        if args.len() != self.function.params.len() {
            return Err(format!(
                "Argument count mismatch: expected {}, got {}",
                self.function.params.len(),
                args.len()
            ));
        }
        for (param_reg, arg_val) in self.function.params.iter().zip(args.into_iter()) {
            self.state.set_reg(*param_reg, arg_val);
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.state.halted {
            return Ok(false);
        }

        let block = self
            .function
            .blocks
            .get(&self.state.pc_block)
            .ok_or_else(|| format!("Invalid block ID {}", self.state.pc_block))?;

        if self.state.pc_inst >= block.instructions.len() {
            return Err(format!(
                "Instruction pointer {} out of bounds for block {}",
                self.state.pc_inst, self.state.pc_block
            ));
        }

        let inst = block.instructions[self.state.pc_inst].clone();
        self.state.pc_inst += 1;

        match inst {
            MirInst::LoadConst(dst, lit) => {
                self.state.set_reg(dst, lit);
            }
            MirInst::Copy(dst, src) => {
                let val = self
                    .state
                    .get_reg(src)
                    .cloned()
                    .ok_or_else(|| format!("Copy from uninitialized register {:?}", src))?;
                self.state.set_reg(dst, val);
            }
            MirInst::BinOp(dst, op, lhs_r, rhs_r) => {
                let lhs = self.state.get_reg_int(lhs_r)?;
                let rhs = self.state.get_reg_int(rhs_r)?;
                let res = match op {
                    MirOp::Add => MirLiteral::Int(lhs.wrapping_add(rhs)),
                    MirOp::Sub => MirLiteral::Int(lhs.wrapping_sub(rhs)),
                    MirOp::Mul => MirLiteral::Int(lhs.wrapping_mul(rhs)),
                    MirOp::Div => {
                        if rhs == 0 {
                            return Err("Division by zero in MIR execution".to_string());
                        }
                        MirLiteral::Int(lhs.wrapping_div(rhs))
                    }
                    MirOp::BitAnd => MirLiteral::Int(lhs & rhs),
                    MirOp::BitOr => MirLiteral::Int(lhs | rhs),
                    MirOp::BitXor => MirLiteral::Int(lhs ^ rhs),
                    MirOp::Eq => MirLiteral::Bool(lhs == rhs),
                    MirOp::Lt => MirLiteral::Bool(lhs < rhs),
                    MirOp::Gt => MirLiteral::Bool(lhs > rhs),
                };
                self.state.set_reg(dst, res);
            }
            MirInst::Alloc(dst, size) => {
                let ptr = self.state.next_alloc_ptr;
                self.state.next_alloc_ptr += (size as u64 + 7) & !7; // 8-byte alignment
                self.state.memory.insert(ptr, vec![0u8; size]);
                self.state.set_reg(dst, MirLiteral::Int(ptr as i64));
            }
            MirInst::Store(ptr_r, offset, val_r) => {
                let ptr = self.state.get_reg_int(ptr_r)? as u64;
                let val = self.state.get_reg_int(val_r)?;
                let mem_block = self
                    .state
                    .memory
                    .get_mut(&ptr)
                    .ok_or_else(|| format!("Invalid memory write at address 0x{:X}", ptr))?;
                if offset + 8 > mem_block.len() {
                    return Err(format!("Buffer overflow on write: offset {} in block size {}", offset, mem_block.len()));
                }
                let bytes = val.to_le_bytes();
                mem_block[offset..offset + 8].copy_from_slice(&bytes);
            }
            MirInst::Load(dst, ptr_r, offset) => {
                let ptr = self.state.get_reg_int(ptr_r)? as u64;
                let mem_block = self
                    .state
                    .memory
                    .get(&ptr)
                    .ok_or_else(|| format!("Invalid memory read at address 0x{:X}", ptr))?;
                if offset + 8 > mem_block.len() {
                    return Err(format!("Buffer overread on load: offset {} in block size {}", offset, mem_block.len()));
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&mem_block[offset..offset + 8]);
                let val = i64::from_le_bytes(buf);
                self.state.set_reg(dst, MirLiteral::Int(val));
            }
            MirInst::Branch(cond_r, then_blk, else_blk) => {
                let cond = self.state.get_reg_bool(cond_r)?;
                self.state.pc_block = if cond { then_blk } else { else_blk };
                self.state.pc_inst = 0;
            }
            MirInst::Jump(target_blk) => {
                self.state.pc_block = target_blk;
                self.state.pc_inst = 0;
            }
            MirInst::Call(dst, fn_name, _) => {
                // Mock intrinsic calls for verifier simulation
                if fn_name == "llvm.assume" {
                    self.state.set_reg(dst, MirLiteral::Int(0));
                } else {
                    return Err(format!("External call {} not supported in isolated interpreter", fn_name));
                }
            }
            MirInst::Return(ret_r) => {
                if let Some(r) = ret_r {
                    self.state.return_value = self.state.get_reg(r).cloned();
                }
                self.state.halted = true;
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn run_to_completion(&mut self, max_steps: usize) -> Result<Option<MirLiteral>, String> {
        let mut steps = 0;
        while !self.state.halted {
            if steps >= max_steps {
                return Err(format!("Execution step limit exceeded ({})", max_steps));
            }
            let active = self.step()?;
            if !active {
                break;
            }
            steps += 1;
        }
        Ok(self.state.return_value.clone())
    }
}
