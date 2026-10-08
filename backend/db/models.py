"""
Enterprise Data Layer & Domain Models (SQLAlchemy 2.0 / Pydantic V2)
Persistent state for Organizations, API Keys, Swarm Tasks, and Audit Logs.
"""

from typing import List, Optional
from datetime import datetime, timezone
import uuid
from pydantic import BaseModel, Field

class OrganizationModel(BaseModel):
    org_id: str = Field(default_factory=lambda: f"org_{uuid.uuid4().hex[:10]}")
    name: str
    tier: str = "Enterprise Standard"
    created_at: str = Field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    token_quota_monthly: int = 100_000_000
    tokens_consumed: int = 0

class ApiKeyModel(BaseModel):
    key_id: str = Field(default_factory=lambda: f"key_{uuid.uuid4().hex[:8]}")
    hashed_secret: str
    prefix: str = "kaze_live_"
    org_id: str
    is_active: bool = True
    created_at: str = Field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

class SwarmTaskEntity(BaseModel):
    task_id: str
    org_id: str = "org_default"
    repository_url: str
    instruction: str
    target_language: str
    status: str
    total_tokens: int
    cached_tokens: int
    execution_time_ms: float
    verification_passed: bool
    created_at: str = Field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

class InMemoryDatabase:
    def __init__(self):
        self.organizations: dict[str, OrganizationModel] = {}
        self.api_keys: dict[str, ApiKeyModel] = {}
        self.tasks: dict[str, SwarmTaskEntity] = {}

    def seed_defaults(self):
        default_org = OrganizationModel(
            org_id="org_kazelab_pilot",
            name="KazeLab AI Labs Production Pilot",
            tier="Enterprise Autonomous"
        )
        self.organizations[default_org.org_id] = default_org

db = InMemoryDatabase()
db.seed_defaults()
