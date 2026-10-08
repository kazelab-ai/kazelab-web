"""
API Key Authentication & Role-Based Access Control (RBAC).
"""

from typing import Optional
from fastapi import HTTPException, Security, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
import hashlib

security_scheme = HTTPBearer(auto_error=False)

class SecurityManager:
    # Pilot master keys
    VALID_HASHES = {
        hashlib.sha256("kaze_live_master_2026".encode()).hexdigest(): "Enterprise Admin",
        hashlib.sha256("kaze_live_pilot_token".encode()).hexdigest(): "Developer Pilot"
    }

    @classmethod
    def verify_api_key(cls, credentials: Optional[HTTPAuthorizationCredentials] = Security(security_scheme)) -> str:
        if not credentials:
            # Allow public sandbox usage for landing page demo
            return "Anonymous Guest"
        token = credentials.credentials
        token_hash = hashlib.sha256(token.encode()).hexdigest()
        if token_hash in cls.VALID_HASHES:
            return cls.VALID_HASHES[token_hash]
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid or expired KazeLab API token"
        )

auth_guard = SecurityManager()
