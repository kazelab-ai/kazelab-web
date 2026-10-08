"""
Async Cluster Connection Pool and Consensus Leader Routing Client for Python.
"""

from typing import List, Dict, Any, Optional
import asyncio
import random


class ClusterLeaderRedirectException(Exception):
    def __init__(self, suggested_leader: Optional[str]):
        super().__init__(f"Request redirected. Active leader: {suggested_leader}")
        self.suggested_leader = suggested_leader


class PyClusterClient:
    """Python asynchronous client for SynapseFlow Raft & SWIM Actor clusters."""

    def __init__(self, seed_endpoints: List[str], auth_token: str):
        self.seed_endpoints = seed_endpoints
        self.auth_token = auth_token
        self.active_leader: Optional[str] = seed_endpoints[0] if seed_endpoints else None
        self.cluster_topology: Dict[str, Dict[str, Any]] = {}
        for idx, ep in enumerate(seed_endpoints):
            self.cluster_topology[ep] = {
                "endpoint": ep,
                "role": "Leader" if idx == 0 else "Follower",
                "term": 1,
                "healthy": True,
            }

    async def propose(self, command: Dict[str, Any], max_retries: int = 3) -> Dict[str, Any]:
        """Proposes a state machine command to the Raft cluster leader with exponential retry."""
        attempt = 0
        while attempt < max_retries:
            attempt += 1
            try:
                if not self.active_leader:
                    raise RuntimeError("No available leader in cluster topology")

                # Mock simulated consensus dispatch
                leader_info = self.cluster_topology.get(self.active_leader)
                if not leader_info or leader_info.get("role") != "Leader":
                    # Find who holds leader role
                    actual_leader = next(
                        (k for k, v in self.cluster_topology.items() if v.get("role") == "Leader"),
                        None,
                    )
                    raise ClusterLeaderRedirectException(actual_leader)

                # Command accepted
                return {
                    "status": "COMMITTED",
                    "term": leader_info["term"],
                    "log_index": random.randint(100, 10000),
                    "leader": self.active_leader,
                    "command_id": command.get("id", "cmd_01"),
                }

            except ClusterLeaderRedirectException as exc:
                if exc.suggested_leader:
                    self.active_leader = exc.suggested_leader
                else:
                    self.active_leader = random.choice(self.seed_endpoints)
                if attempt >= max_retries:
                    raise
                await asyncio.sleep(0.05 * (2 ** attempt))

        raise RuntimeError("Consensus proposal timed out after maximum retries")
