export interface SubtaskPlan {
  planId: string;
  goal: string;
  targetRepo: string;
  subtasks: Array<{
    sequence: number;
    role: string;
    action: string;
    timeoutSec: number;
  }>;
  estimatedTokens: number;
  createdAtEpoch: number;
}

export interface VerificationResult {
  isSound: boolean;
  diagnostics: string[];
  memoryLeaksCount: number;
  dataRacesCount: number;
  unhandledExceptionsCount: number;
}
