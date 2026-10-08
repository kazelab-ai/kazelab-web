"""
V2 Modular API Router with Plan Generation and Prover Verification.
"""

from fastapi import APIRouter, Depends
from typing import Dict, Any
from core.orchestrator.planner import orchestrator_v4, SubtaskPlan
from core.verification.prover import prover_harness, VerificationResult
from security.auth import auth_guard

router_v2 = APIRouter(prefix="/api/v2", tags=["V2 Enterprise Swarm"])

@router_v2.post("/planner/decompose", response_model=SubtaskPlan)
async def decompose_plan(
    goal: str,
    target_repo: str,
    user_role: str = Depends(auth_guard.verify_api_key)
) -> SubtaskPlan:
    return orchestrator_v4.generate_plan(goal=goal, repo=target_repo)

@router_v2.post("/prover/evaluate", response_model=VerificationResult)
async def evaluate_patch_soundness(
    patch_code: str,
    user_role: str = Depends(auth_guard.verify_api_key)
) -> VerificationResult:
    return prover_harness.evaluate_patch(patch_code=patch_code)
