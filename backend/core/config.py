from pydantic_settings import BaseSettings, SettingsConfigDict
from pydantic import Field
from typing import List

class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    app_name: str = "KazeLab AI - SynapseFlow Engine"
    app_version: str = "3.0.0"
    environment: str = "production"
    host: str = "0.0.0.0"
    port: int = 8000
    
    # Model & AI Engine Configuration
    primary_model: str = "claude-3-5-sonnet-20241022"
    anthropic_api_key: str = ""
    prompt_caching_ttl_seconds: int = 300
    max_reasoning_tokens: int = 8192
    
    # CORS
    allowed_origins: List[str] = [
        "https://kazelab.xyz",
        "https://www.kazelab.xyz",
        "http://localhost:3000",
        "http://localhost:8000",
        "http://127.0.0.1:3000",
        "http://127.0.0.1:8000",
        "http://localhost:5500",
        "*"
    ]
    
    # Enterprise & Founder
    founder_email: str = "founder@kazelab.xyz"
    company_name: str = "KazeLab AI Labs"
    official_website: str = "https://kazelab.xyz"

settings = Settings()
