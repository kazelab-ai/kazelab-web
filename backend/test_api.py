import pytest
from httpx import AsyncClient, ASGITransport
from main import app

@pytest.mark.asyncio
async def test_health_check():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/health")
        assert res.status_code == 200
        data = res.json()
        assert data["status"] == "healthy"
        assert data["version"] == "3.0.0"
        assert "Claude 3.5 Sonnet" in data["foundation_engine"]

@pytest.mark.asyncio
async def test_telemetry():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/telemetry")
        assert res.status_code == 200
        data = res.json()
        assert data["swe_bench_verified"] == 94.8
        assert data["mcp_cluster_nodes_online"] == 8

@pytest.mark.asyncio
async def test_mcp_hub_registry():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/api/v1/mcp/servers")
        assert res.status_code == 200
        servers = res.json()
        assert len(servers) >= 4
        assert any(s["server_id"] == "mcp-ast-analyzer" for s in servers)

@pytest.mark.asyncio
async def test_swarm_dispatch_cycle():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        payload = {
            "repository_url": "https://github.com/kazelab/distributed-runtime",
            "instruction": "Refactor memory allocation to jemalloc with zero false sharing",
            "target_language": "C++20",
            "enable_prompt_caching": True
        }
        res = await client.post("/api/v1/swarm/dispatch", json=payload)
        assert res.status_code == 202
        data = res.json()
        assert data["status"] == "COMPLETED"
        assert data["tokens_saved_via_cache"] > 0
        assert len(data["steps_executed"]) == 4

        # Retrieve task
        task_id = data["task_id"]
        res_get = await client.get(f"/api/v1/swarm/task/{task_id}")
        assert res_get.status_code == 200
        assert res_get.json()["task_id"] == task_id

@pytest.mark.asyncio
async def test_lead_and_waitlist_pipeline():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.post("/api/v1/waitlist/join", json={"email": "architect@silicon-valley-ai.org"})
        assert res.status_code == 201
        data = res.json()
        assert data["success"] is True
        assert data["queue_position"] >= 142
