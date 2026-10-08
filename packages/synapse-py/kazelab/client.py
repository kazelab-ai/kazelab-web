"""
Official Python Client SDK for KazeLab SynapseFlow
Enables AI researchers and backend developers to programmatically control agent swarms.
"""

from typing import Dict, Any, Optional, List
import httpx

class SynapseFlowClient:
    """Client for interacting with KazeLab SynapseFlow Engine."""

    def __init__(self, base_url: str = "https://kazelab.xyz", api_key: Optional[str] = None):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self._headers = {"Content-Type": "application/json"}
        if self.api_key:
            self._headers["Authorization"] = f"Bearer {self.api_key}"

    def get_health(self) -> Dict[str, Any]:
        with httpx.Client(base_url=self.base_url) as client:
            res = client.get("/health")
            res.raise_for_status()
            return res.json()

    def get_telemetry(self) -> Dict[str, Any]:
        with httpx.Client(base_url=self.base_url) as client:
            res = client.get("/api/v1/telemetry")
            res.raise_for_status()
            return res.json()

    def dispatch_swarm(
        self,
        repository_url: str,
        instruction: str,
        target_language: str = "Rust",
        enable_prompt_caching: bool = True
    ) -> Dict[str, Any]:
        payload = {
            "repository_url": repository_url,
            "instruction": instruction,
            "target_language": target_language,
            "enable_prompt_caching": enable_prompt_caching
        }
        with httpx.Client(base_url=self.base_url) as client:
            res = client.post("/api/v1/swarm/dispatch", json=payload, headers=self._headers)
            res.raise_for_status()
            return res.json()
