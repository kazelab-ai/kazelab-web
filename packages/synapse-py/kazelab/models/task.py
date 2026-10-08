"""
Synapse Python Models for Swarm and Telemetry.
"""

from pydantic import BaseModel
from typing import List, Optional

class SwarmTaskResponse(BaseModel):
    task_id: str
    status: str
    prompt_cache_hit_rate: str
    total_tokens_consumed: int
    tokens_saved_via_cache: int
    synthesized_patch: str
    verification_passed: bool
