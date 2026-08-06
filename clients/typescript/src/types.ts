export type Operation = "remember" | "recall" | "improve" | "forget";

// The wire matches crates/marciana-memory/src/api.rs exactly: the server
// denies unknown fields, so these shapes carry no client-only extras.
export interface RememberRequest {
  space_id: string;
  text: string;
  purpose: string;
}

export interface RecallRequest {
  space_id: string;
  query: string;
  purpose: string;
}

export interface ImproveRequest {
  space_id: string;
  memory_id: string;
  replacement: RememberRequest;
}

export interface ForgetRequest {
  space_id: string;
  memory_ids: string[];
  purpose: string;
}

export interface MemoryReceipt {
  operation: Operation;
  allowed: boolean;
  memory_ids: string[];
  detail?: string;
}

const identity = /^[A-Za-z0-9_:/.-]+$/;

function isIdentity(value: string): boolean {
  return value.length > 0 && value.length <= 256 && identity.test(value);
}

export function validateRequest(request: RememberRequest | RecallRequest | ImproveRequest | ForgetRequest): void {
  const value = request as { space_id: string; purpose?: string; text?: string; query?: string; memory_id?: string };
  for (const field of [value.space_id, value.purpose, value.memory_id]) {
    if (field !== undefined && !isIdentity(field)) throw new Error("invalid memory identity");
  }
  const text = value.text ?? value.query;
  if (text !== undefined && (!text || text.length > 16384)) throw new Error("invalid memory text");
  if ("memory_ids" in request) {
    if (!request.memory_ids.length || request.memory_ids.length > 256) throw new Error("invalid memory ids");
    for (const id of request.memory_ids) {
      if (!isIdentity(id)) throw new Error("invalid memory identity");
    }
  }
  if ("replacement" in request) validateRequest(request.replacement);
}
