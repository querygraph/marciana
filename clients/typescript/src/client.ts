import {
  ForgetRequest,
  ImproveRequest,
  MemoryReceipt,
  RecallRequest,
  RememberRequest,
  Operation,
  validateRequest,
} from "./types.js";

export interface MemoryTransport {
  post(path: string, payload: unknown): Promise<MemoryReceipt>;
}

export class MarcianaClient {
  public constructor(private readonly transport: MemoryTransport) {}

  public remember(request: RememberRequest): Promise<MemoryReceipt> {
    return this.call("/v1/memory/remember", "remember", request);
  }

  public recall(request: RecallRequest): Promise<MemoryReceipt> {
    return this.call("/v1/memory/recall", "recall", request);
  }

  public improve(request: ImproveRequest): Promise<MemoryReceipt> {
    return this.call("/v1/memory/improve", "improve", request);
  }

  public forget(request: ForgetRequest): Promise<MemoryReceipt> {
    return this.call("/v1/memory/forget", "forget", request);
  }

  private async call(path: string, operation: Operation, request: RememberRequest | RecallRequest | ImproveRequest | ForgetRequest): Promise<MemoryReceipt> {
    validateRequest(request);
    const receipt = await this.transport.post(path, request);
    if (receipt.operation !== operation || !Array.isArray(receipt.memoryIds)) throw new Error("invalid memory receipt");
    return receipt;
  }
}
