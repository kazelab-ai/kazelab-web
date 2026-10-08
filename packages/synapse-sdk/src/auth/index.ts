export interface AuthTokenPayload {
  token: string;
  role: 'Enterprise Admin' | 'Developer Pilot' | 'Anonymous Guest';
  expiresAtEpoch: number;
}

export class AuthenticationManager {
  private currentToken?: string;

  constructor(token?: string) {
    this.currentToken = token;
  }

  public setToken(token: string): void {
    this.currentToken = token;
  }

  public getAuthorizationHeader(): Record<string, string> {
    if (!this.currentToken) return {};
    return { Authorization: `Bearer ${this.currentToken}` };
  }
}
