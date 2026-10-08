//! Constant Propagation and Dead Code Elimination Optimization Passes for MIR.
//! Evaluates static expressions and eliminates unreachable basic blocks.

use crate::mir::interpreter::{MirFunction, MirInst, MirLiteral, MirOp, Reg};
use std::collections::{HashMap, HashSet};

pub struct MirOptimizationPipeline;

impl MirOptimizationPipeline {
    pub fn run_passes(mut func: MirFunction) -> MirFunction {
        func = Self::constant_propagation(func);
        func = Self::dead_block_elimination(func);
        func
    }

    pub fn constant_propagation(mut func: MirFunction) -> MirFunction {
        let mut const_regs: HashMap<Reg, MirLiteral> = HashMap::new();

        for block in func.blocks.values_mut() {
            let mut optimized_instructions = Vec::new();

            for inst in block.instructions.drain(..) {
                match inst {
                    MirInst::LoadConst(dst, ref lit) => {
                        const_regs.insert(dst, lit.clone());
                        optimized_instructions.push(inst);
                    }
                    MirInst::BinOp(dst, op, lhs_r, rhs_r) => {
                        if let (Some(MirLiteral::Int(l)), Some(MirLiteral::Int(r))) =
                            (const_regs.get(&lhs_r), const_regs.get(&rhs_r))
                        {
                            let folded = match op {
                                MirOp::Add => MirLiteral::Int(l.wrapping_add(*r)),
                                MirOp::Sub => MirLiteral::Int(l.wrapping_sub(*r)),
                                MirOp::Mul => MirLiteral::Int(l.wrapping_mul(*r)),
                                MirOp::Div if *r != 0 => MirLiteral::Int(l.wrapping_div(*r)),
                                MirOp::BitAnd => MirLiteral::Int(l & r),
                                MirOp::BitOr => MirLiteral::Int(l | r),
                                MirOp::BitXor => MirLiteral::Int(l ^ r),
                                MirOp::Eq => MirLiteral::Bool(l == r),
                                MirOp::Lt => MirLiteral::Bool(l < r),
                                MirOp::Gt => MirLiteral::Bool(l > r),
                                _ => {
                                    optimized_instructions.push(MirInst::BinOp(dst, op, lhs_r, rhs_r));
                                    continue;
                                }
                            };
                            const_regs.insert(dst, folded.clone());
                            optimized_instructions.push(MirInst::LoadConst(dst, folded));
                        } else {
                            optimized_instructions.push(MirInst::BinOp(dst, op, lhs_r, rhs_r));
                        }
                    }
                    other => optimized_instructions.push(other),
                }
            }
            block.instructions = optimized_instructions;
        }

        func
    }

    pub fn dead_block_elimination(mut func: MirFunction) -> MirFunction {
        let mut reachable = HashSet::new();
        let mut queue = vec![func.entry_block];
        reachable.insert(func.entry_block);

        while let Some(blk_id) = queue.pop() {
            if let Some(block) = func.blocks.get(&blk_id) {
                for inst in &block.instructions {
                    match inst {
                        MirInst::Branch(_, then_b, else_b) => {
                            if reachable.insert(*then_b) {
                                queue.push(*then_b);
                            }
                            if reachable.insert(*else_b) {
                                queue.push(*else_b);
                            }
                        }
                        MirInst::Jump(target) => {
                            if reachable.insert(*target) {
                                queue.push(*target);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        func.blocks.retain(|id, _| reachable.contains(id));
        func
    }
}
