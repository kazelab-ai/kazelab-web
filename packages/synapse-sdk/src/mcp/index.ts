export interface McpToolDescriptor {
  name: string;
  description: string;
  schema: Record<string, unknown>;
  isSandboxed: boolean;
}

export class McpClientProtocol {
  private registeredTools: Map<string, McpToolDescriptor> = new Map();

  public registerTool(tool: McpToolDescriptor): void {
    this.registeredTools.set(tool.name, tool);
  }

  public getTool(name: string): McpToolDescriptor | undefined {
    return this.registeredTools.get(name);
  }

  public listTools(): McpToolDescriptor[] {
    return Array.from(this.registeredTools.values());
  }
}
