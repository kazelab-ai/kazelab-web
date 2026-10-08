"""
PDDL (Planning Domain Definition Language) Forward Search Engine.
Converts high-level refactoring goals into verifiable STRIPS action sequences.
Reference: Fikes & Nilsson (1971), Ghallab et al. (2004).
"""

from typing import List, Set, Dict, Any, Optional
from pydantic import BaseModel

class GroundedPredicate(BaseModel):
    name: str
    target: str

    def __hash__(self):
        return hash((self.name, self.target))

    def __eq__(self, other):
        return self.name == other.name and self.target == other.target

class PddlEngineeringAction(BaseModel):
    action_name: str
    preconditions: List[GroundedPredicate]
    add_effects: List[GroundedPredicate]
    delete_effects: List[GroundedPredicate]

class ForwardSearchPlanner:
    def __init__(self, actions: List[PddlEngineeringAction]):
        self.actions = actions

    def solve(
        self,
        initial_state: Set[GroundedPredicate],
        goal_state: Set[GroundedPredicate],
        max_horizon: int = 10
    ) -> Optional[List[str]]:
        state = set(initial_state)
        plan: List[str] = []

        for _ in range(max_horizon):
            if goal_state.issubset(state):
                return plan

            matched = False
            for act in self.actions:
                if set(act.preconditions).issubset(state):
                    for d in act.delete_effects:
                        state.discard(d)
                    for a in act.add_effects:
                        state.add(a)
                    plan.append(act.action_name)
                    matched = True
                    break

            if not matched:
                break

        return plan if goal_state.issubset(state) else None

# Default Software Engineering Domain Actions
DEFAULT_SW_ACTIONS = [
    PddlEngineeringAction(
        action_name="parse_ast_and_cache",
        preconditions=[GroundedPredicate(name="repo_mounted", target="target_repo")],
        add_effects=[GroundedPredicate(name="ast_cached", target="target_repo")],
        delete_effects=[]
    ),
    PddlEngineeringAction(
        action_name="synthesize_raii_patch",
        preconditions=[GroundedPredicate(name="ast_cached", target="target_repo")],
        add_effects=[GroundedPredicate(name="patch_drafted", target="target_repo")],
        delete_effects=[]
    ),
    PddlEngineeringAction(
        action_name="run_sandbox_verifier",
        preconditions=[GroundedPredicate(name="patch_drafted", target="target_repo")],
        add_effects=[GroundedPredicate(name="tests_passed", target="target_repo")],
        delete_effects=[]
    ),
    PddlEngineeringAction(
        action_name="verify_cve_and_memory_safety",
        preconditions=[GroundedPredicate(name="tests_passed", target="target_repo")],
        add_effects=[GroundedPredicate(name="production_ready", target="target_repo")],
        delete_effects=[]
    )
]

pddl_planner = ForwardSearchPlanner(DEFAULT_SW_ACTIONS)
