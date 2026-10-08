import { SystemTelemetry } from '../index';

export class TelemetryStreamHandler {
  private listeners: Array<(data: SystemTelemetry) => void> = [];

  public subscribe(listener: (data: SystemTelemetry) => void): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  public emit(data: SystemTelemetry): void {
    for (const listener of this.listeners) {
      listener(data);
    }
  }
}
