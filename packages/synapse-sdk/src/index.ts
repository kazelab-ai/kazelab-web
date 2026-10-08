/**
 * @kazelab/synapse-sdk — Official TypeScript SDK for KazeLab SynapseFlow
 * Real-time Multi-Agent Swarm Orchestration, MCP Tool Call Streaming, and AST Ingestion
 */

export interface SwarmTaskRequest {
  repositoryUrl: string;
  instruction: string;
  targetLanguage?: 'Rust' | 'C++20' | 'Go' | 'Python' | 'TypeScript';
  enablePromptCaching?: boolean;
  activeMcpServers?: string[];
  maxReasoningCycles?: number;
}

export interface ExecutionStep {
  stepNumber: number;
  agentRole: 'Cognitive Architect' | 'Systems Coder' | 'Verification Engine' | 'Security & Evasion Auditor';
  action: string;
  toolInvoked?: string;
  status: 'PENDING' | 'RUNNING' | 'SUCCESS' | 'FAILED';
  durationMs: number;
  outputSummary: string;
  tokensUsed: number;
  cachedTokens: number;
  timestamp: string;
}

export interface SwarmTaskResult {
  taskId: string;
  status: 'COMPLETED' | 'FAILED' | 'IN_PROGRESS';
  modelBrain: string;
  promptCacheHitRate: string;
  totalTokensConsumed: number;
  tokensSavedViaCache: number;
  executionTimeMs: number;
  synthesizedPatch: string;
  verificationPassed: boolean;
  securityAuditClean: boolean;
  stepsExecuted: ExecutionStep[];
  createdAt: string;
}

export interface SystemTelemetry {
  serviceStatus: string;
  activeEngine: string;
  protocol: string;
  sweBenchVerified: number;
  promptCachingEfficiency: string;
  averageTtftMs: number;
  mcpClusterNodesOnline: number;
  totalPatchesSynthesized: number;
}

export class SynapseClient {
  private baseUrl: string;
  private apiKey?: string;

  constructor(options: { baseUrl?: string; apiKey?: string } = {}) {
    this.baseUrl = options.baseUrl || 'https://kazelab.xyz';
    this.apiKey = options.apiKey;
  }

  /**
   * Health check and node status
   */
  async getHealth(): Promise<{ status: string; version: string; foundation_engine: string }> {
    const res = await fetch(`${this.baseUrl}/health`);
    if (!res.ok) throw new Error(`Health check failed: ${res.statusText}`);
    return res.json();
  }

  /**
   * Fetch real-time cluster telemetry
   */
  async getTelemetry(): Promise<SystemTelemetry> {
    const res = await fetch(`${this.baseUrl}/api/v1/telemetry`);
    if (!res.ok) throw new Error(`Telemetry fetch failed: ${res.statusText}`);
    return res.json();
  }

  /**
   * Dispatch an autonomous multi-agent swarm task
   */
  async dispatchTask(request: SwarmTaskRequest): Promise<SwarmTaskResult> {
    const res = await fetch(`${this.baseUrl}/api/v1/swarm/dispatch`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(this.apiKey ? { Authorization: `Bearer ${this.apiKey}` } : {}),
      },
      body: JSON.stringify({
        repository_url: request.repositoryUrl,
        instruction: request.instruction,
        target_language: request.targetLanguage || 'Rust',
        enable_prompt_caching: request.enablePromptCaching ?? true,
        active_mcp_servers: request.activeMcpServers || [
          'mcp-ast-analyzer',
          'mcp-sandbox-runner',
          'mcp-git-engine',
          'mcp-security-auditor',
        ],
        max_reasoning_cycles: request.maxReasoningCycles || 5,
      }),
    });

    if (!res.ok) {
      const err = await res.text();
      throw new Error(`Failed to dispatch swarm task: ${err}`);
    }

    return res.json();
  }

  /**
   * Open real-time WebSocket telemetry pipe
   */
  connectTelemetryStream(onData: (data: SystemTelemetry) => void): WebSocket {
    const wsUrl = this.baseUrl.replace(/^http/, 'ws') + '/ws/telemetry';
    const ws = new WebSocket(wsUrl);
    ws.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        onData(parsed);
      } catch (err) {
        console.error('Failed to parse telemetry frame:', err);
      }
    };
    return ws;
  }
}
