export interface ActiveSession {
  id: string;
  label: string;
  project?: string;
  needsAttention: boolean;
  lastActivityAt: number;
  role?: string;
  elapsed?: string;
  meta?: string;
}
