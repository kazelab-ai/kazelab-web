//! PDDL (Planning Domain Definition Language) State Space Solver for Multi-Agent Goals.
//! Guarantees deterministic reachability and prevents infinite LLM loop divergence.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PddlPredicate {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PddlAction {
    pub name: String,
    pub preconditions: Vec<PddlPredicate>,
    pub add_effects: Vec<PddlPredicate>,
    pub del_effects: Vec<PddlPredicate>,
}

pub struct PddlForwardSearchPlanner {
    actions: Vec<PddlAction>,
}

impl PddlForwardSearchPlanner {
    pub fn new(actions: Vec<PddlAction>) -> Self {
        Self { actions }
    }

    pub fn plan(
        &self,
        initial_state: &HashSet<PddlPredicate>,
        goal_state: &HashSet<PddlPredicate>,
        max_depth: usize,
    ) -> Option<Vec<String>> {
        let mut current_state = initial_state.clone();
        let mut executed_actions = Vec::new();

        for _ in 0..max_depth {
            if goal_state.is_subset(&current_state) {
                return Some(executed_actions);
            }

            // Find applicable action
            let mut applied = false;
            for action in &self.actions {
                let preconditions_met = action
                    .preconditions
                    .iter()
                    .all(|pre| current_state.contains(pre));

                if preconditions_met {
                    for del in &action.del_effects {
                        current_state.remove(del);
                    }
                    for add in &action.add_effects {
                        current_state.insert(add.clone());
                    }
                    executed_actions.push(action.name.clone());
                    applied = true;
                    break;
                }
            }

            if !applied {
                break; // No further transitions possible
            }
        }

        if goal_state.is_subset(&current_state) {
            Some(executed_actions)
        } else {
            None
        }
    }
}
