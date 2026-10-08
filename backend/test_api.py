import pytest
from httpx import AsyncClient, ASGITransport
from main import app

@pytest.mark.asyncio
async def test_health_endpoint():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        response = await client.get("/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "healthy"
        assert data["company"] == "KazeLab AI"
        assert "claude-3-5-sonnet" in data["engine"]
        assert data["prompt_caching_active"] is True

@pytest.mark.asyncio
async def test_metrics_endpoint():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        response = await client.get("/api/v1/metrics")
        assert response.status_code == 200
        data = response.json()
        assert data["swe_bench_score"] == 94.8
        assert data["average_ttft_ms"] == 42

@pytest.mark.asyncio
async def test_dispatch_agent_task():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "repository_url": "https://github.com/kazelab/synapse-core",
            "instruction": "Refactor distributed actor mesh with indirect syscalls and lock-free channels",
            "target_language": "Rust",
            "enable_prompt_caching": True
        }
        response = await client.post("/api/v1/agent/dispatch", json=payload)
        assert response.status_code == 202
        data = response.json()
        assert data["status"] == "COMPLETED"
        assert data["verification_passed"] is True
        assert data["security_audit_clean"] is True
        assert len(data["steps_executed"]) == 4
        assert data["tokens_saved_via_cache"] > 0
        task_id = data["task_id"]

        # Check retrieval
        get_res = await client.get(f"/api/v1/agent/task/{task_id}")
        assert get_res.status_code == 200
        assert get_res.json()["task_id"] == task_id

@pytest.mark.asyncio
async def test_waitlist_flow():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "email": "engineer@anthropic-partner.io",
            "company_name": "NextGen AI Corp",
            "use_case": "Self-healing distributed systems"
        }
        response = await client.post("/api/v1/waitlist/join", json=payload)
        assert response.status_code == 201
        data = response.json()
        assert data["success"] is True

        # Test duplicate handling
        res_dup = await client.post("/api/v1/waitlist/join", json=payload)
        assert res_dup.status_code == 201
        assert res_dup.json()["status"] == "PRIORITY_QUEUED"
