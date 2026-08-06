export type MemoryKind = "semantic" | "episodic" | "profile" | "event";
export type Clearance = "public" | "internal" | "sensitive" | "secret";
export type ForgetMode = "erase" | "retract";
export type Operation = "remember" | "recall" | "improve" | "forget";

export interface RememberRequest {
  space: string;
  text: string;
  purpose: string;
  kind?: MemoryKind;
}

export interface RecallRequest {
  space: string;
  query: string;
  purpose: string;
  clearance?: Clearance;
}

export interface ImproveRequest {
  space: string;
  memory_id: string;
  replacement: RememberRequest;
}

export interface ForgetRequest {
  space: string;
  ids: string[];
  purpose: string;
  mode?: ForgetMode;
}

export interface MemoryReceipt {
  operation: Operation;
  allowed: boolean;
  memory_ids: string[];
  detail?: string;
}

const identity = /^[A-Za-z0-9_:/.-]+$/;

export function validateRequest(request: RememberRequest | RecallRequest | ImproveRequest | ForgetRequest): void {
  const value = request as { space: string; purpose?: string; text?: string; query?: string; memory_id?: string };
  for (const field of [value.space, value.purpose, value.memory_id]) {
    if (field !== undefined && (!field || field.length > 256 || !identity.test(field))) {
      throw new Error("invalid memory identity");
    }
  }
  const text = value.text ?? value.query;
  if (text !== undefined && (!text || text.length > 16384)) throw new Error("invalid memory text");
  if ("ids" in request && (!request.ids.length || request.ids.length > 256)) throw new Error("invalid memory ids");
}
