//! Mid-Level Intermediate Representation (MIR) Basic Block Lowering and SSA Form Generation.
//! Converts High-Level AST expressions into basic blocks, control flow edges, and virtual register assignments.

use crate::mir::interpreter::{MirBasicBlock, MirFunction, MirInst, MirLiteral, MirOp, Reg};
use std::collections::HashMap;

pub struct MirBuilder {
    next_reg_id: u32,
    next_block_id: usize,
    blocks: HashMap<usize, MirBasicBlock>,
    current_block: usize,
}

impl MirBuilder {
    pub fn new() -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            0,
            MirBasicBlock {
                id: 0,
                instructions: Vec::new(),
            },
        );
        Self {
            next_reg_id: 0,
            next_block_id: 1,
            blocks,
            current_block: 0,
        }
    }

    pub fn alloc_reg(&mut self) -> Reg {
        let r = Reg(self.next_reg_id);
        self.next_reg_id += 1;
        r
    }

    pub fn create_block(&mut self) -> usize {
        let id = self.next_block_id;
        self.next_block_id += 1;
        self.blocks.insert(
            id,
            MirBasicBlock {
                id,
                instructions: Vec::new(),
            },
        );
        id
    }

    pub fn switch_to_block(&mut self, block_id: usize) {
        self.current_block = block_id;
    }

    pub fn emit(&mut self, inst: MirInst) {
        if let Some(block) = self.blocks.get_mut(&self.current_block) {
            block.instructions.push(inst);
        }
    }

    pub fn emit_load_const_int(&mut self, val: i64) -> Reg {
        let r = self.alloc_reg();
        self.emit(MirInst::LoadConst(r, MirLiteral::Int(val)));
        r
    }

    pub fn emit_load_const_bool(&mut self, val: bool) -> Reg {
        let r = self.alloc_reg();
        self.emit(MirInst::LoadConst(r, MirLiteral::Bool(val)));
        r
    }

    pub fn emit_add(&mut self, lhs: Reg, rhs: Reg) -> Reg {
        let r = self.alloc_reg();
        self.emit(MirInst::BinOp(r, MirOp::Add, lhs, rhs));
        r
    }

    pub fn emit_sub(&mut self, lhs: Reg, rhs: Reg) -> Reg {
        let r = self.alloc_reg();
        self.emit(MirInst::BinOp(r, MirOp::Sub, lhs, rhs));
        r
    }

    pub fn emit_mul(&mut self, lhs: Reg, rhs: Reg) -> Reg {
        let r = self.alloc_reg();
        self.emit(MirInst::BinOp(r, MirOp::Mul, lhs, rhs));
        r
    }

    pub fn emit_branch(&mut self, cond: Reg, then_blk: usize, else_blk: usize) {
        self.emit(MirInst::Branch(cond, then_blk, else_blk));
    }

    pub fn emit_jump(&mut self, target_blk: usize) {
        self.emit(MirInst::Jump(target_blk));
    }

    pub fn emit_return(&mut self, val: Option<Reg>) {
        self.emit(MirInst::Return(val));
    }

    pub fn finish(self, name: &str, params: Vec<Reg>) -> MirFunction {
        MirFunction {
            name: name.to_string(),
            params,
            blocks: self.blocks,
            entry_block: 0,
        }
    }
}
