"""
Waitlist & Enterprise Partnership Inquiries Service
"""

from typing import List, Dict, Any, Optional
import uuid
from datetime import datetime, timezone
from pydantic import BaseModel, EmailStr, Field

class WaitlistSubmission(BaseModel):
    email: EmailStr
    company_name: Optional[str] = "Independent Dev"
    use_case: Optional[str] = "Enterprise Autonomous Code Synthesis"
    organization_size: Optional[str] = "1-10"
    created_at: str = Field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

class ContactInquiry(BaseModel):
    name: str = Field(..., min_length=2)
    email: EmailStr
    subject: str = Field(..., min_length=3)
    message: str = Field(..., min_length=10)
    created_at: str = Field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

class LeadManagementService:
    def __init__(self):
        self._waitlist: List[WaitlistSubmission] = []
        self._contacts: List[ContactInquiry] = []

    def enroll_waitlist(self, submission: WaitlistSubmission) -> Dict[str, Any]:
        for existing in self._waitlist:
            if existing.email.lower() == submission.email.lower():
                return {
                    "success": True,
                    "message": "Welcome back! Your priority access registration is confirmed.",
                    "email": submission.email,
                    "queue_position": 142
                }
        self._waitlist.append(submission)
        return {
            "success": True,
            "message": "Priority access confirmed! Welcome to KazeLab SynapseFlow Early Access.",
            "email": submission.email,
            "queue_position": len(self._waitlist) + 142
        }

    def record_contact(self, contact: ContactInquiry) -> Dict[str, Any]:
        self._contacts.append(contact)
        return {
            "success": True,
            "message": f"Inquiry registered. Founder team (founder@kazelab.xyz) will respond to {contact.email} within 24h.",
            "inquiry_id": f"inq_{uuid.uuid4().hex[:8]}"
        }

    def total_leads(self) -> int:
        return len(self._waitlist)

lead_service = LeadManagementService()
