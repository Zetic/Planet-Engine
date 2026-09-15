import {
  WORLDGEN_PROTOCOL_VERSION,
  validateClimateRequest,
  validateDrainageRequest,
  validateGeologyRequest,
  validateInheritanceRequest,
  validateLithosphereRequest,
  validateSyntheticRequest,
  validateTopographyRequest,
  validateTectonicsRequest,
  validateTopologyRequest,
  worldgenClimateCommand,
  worldgenDrainageCommand,
  worldgenGeologyCommand,
  worldgenInheritanceCommand,
  worldgenLithosphereCommand,
  worldgenSyntheticCommand,
  worldgenTopographyCommand,
  worldgenTectonicsCommand,
  worldgenTopologyCommand,
  type WorldgenClimateRequest,
  type WorldgenClimateResult,
  type WorldgenDrainageRequest,
  type WorldgenDrainageResult,
  type WorldgenEvent,
  type WorldgenGenerationProgress,
  type WorldgenGeologyRequest,
  type WorldgenGeologyResult,
  type WorldgenInheritanceRequest,
  type WorldgenInheritanceResult,
  type WorldgenLithosphereRequest,
  type WorldgenLithosphereResult,
  type WorldgenSyntheticRequest,
  type WorldgenTopographyRequest,
  type WorldgenTopographyResult,
  type WorldgenSyntheticResult,
  type WorldgenTectonicsRequest,
  type WorldgenTectonicsResult,
  type WorldgenTopologyRequest,
  type WorldgenTopologyResult,
} from './protocol.js';

type WorldgenResult = WorldgenSyntheticResult | WorldgenTopologyResult | WorldgenTectonicsResult | WorldgenGeologyResult | WorldgenLithosphereResult | WorldgenInheritanceResult | WorldgenTopographyResult | WorldgenClimateResult | WorldgenDrainageResult;
type WorldgenRequestCommand = ReturnType<typeof worldgenSyntheticCommand> | ReturnType<typeof worldgenTopologyCommand> | ReturnType<typeof worldgenTectonicsCommand> | ReturnType<typeof worldgenGeologyCommand> | ReturnType<typeof worldgenLithosphereCommand> | ReturnType<typeof worldgenInheritanceCommand> | ReturnType<typeof worldgenTopographyCommand> | ReturnType<typeof worldgenClimateCommand> | ReturnType<typeof worldgenDrainageCommand>;
interface PendingRequest { resolve: (result: WorldgenResult) => void; reject: (error: Error) => void; progress?: (progress: WorldgenGenerationProgress) => void; commandType: string; }

export interface WorldgenClientDiagnosticEvent {
  event: string;
  requestId?: number;
  commandType?: string;
  details?: Record<string, unknown>;
}
export interface WorldgenClientOptions { onDiagnostic?: (diagnostic: WorldgenClientDiagnosticEvent) => void; }

export interface WorldgenClient {
  generateSynthetic(request: WorldgenSyntheticRequest): Promise<WorldgenSyntheticResult>;
  generateTopology(request: WorldgenTopologyRequest): Promise<WorldgenTopologyResult>;
  generateTectonics(request: WorldgenTectonicsRequest): Promise<WorldgenTectonicsResult>;
  generateGeology(request: WorldgenGeologyRequest): Promise<WorldgenGeologyResult>;
  generateLithosphere(request: WorldgenLithosphereRequest): Promise<WorldgenLithosphereResult>;
  generateInheritance(request: WorldgenInheritanceRequest): Promise<WorldgenInheritanceResult>;
  generateTopography(request: WorldgenTopographyRequest): Promise<WorldgenTopographyResult>;
  generateClimate(request: WorldgenClimateRequest, onProgress?: (progress: WorldgenGenerationProgress) => void): Promise<WorldgenClimateResult>;
  generateDrainage(request: WorldgenDrainageRequest): Promise<WorldgenDrainageResult>;
  dispose(): void;
}

export function createWorldgenClient(options: WorldgenClientOptions = {}): WorldgenClient {
  const workerUrl = new URL('./worldgenWorker.js', import.meta.url);
  workerUrl.searchParams.set('v', String(WORLDGEN_PROTOCOL_VERSION));
  const pending = new Map<number, PendingRequest>();
  let nextRequestId = 1;
  let disposed = false;
  let recycleWhenIdle = false;
  let workerGeneration = 0;
  let worker: Worker;

  const diagnostic = (event: string, requestId?: number, commandType?: string, details?: Record<string, unknown>): void => {
    try { options.onDiagnostic?.({ event, requestId, commandType, details }); } catch { /* diagnostics must never break generation */ }
  };

  function rejectAll(message: string, source = 'worker-reject-all'): void {
    diagnostic(source, undefined, undefined, { message, pendingCount: pending.size, workerGeneration });
    for (const request of pending.values()) request.reject(new Error(message));
    pending.clear();
  }

  function spawnWorker(): Worker {
    const generation = ++workerGeneration;
    const next = new Worker(workerUrl, { type: 'module' });
    diagnostic('worker-spawned', undefined, undefined, { workerGeneration: generation, workerUrl: workerUrl.toString() });
    next.addEventListener('message', (event: MessageEvent<WorldgenEvent>) => {
      if (next !== worker) {
        diagnostic('stale-worker-message', undefined, undefined, { workerGeneration: generation });
        return;
      }
      const message = event.data;
      if (!message) {
        diagnostic('empty-worker-message', undefined, undefined, { workerGeneration: generation });
        return;
      }
      if (message.protocolVersion !== WORLDGEN_PROTOCOL_VERSION) {
        diagnostic('protocol-mismatch', message.requestId, message.type, { expected: WORLDGEN_PROTOCOL_VERSION, actual: message.protocolVersion, workerGeneration: generation });
        return;
      }
      const request = pending.get(message.requestId);
      if (!request) {
        diagnostic('orphan-worker-message', message.requestId, message.type, { workerGeneration: generation });
        return;
      }
      diagnostic('worker-message-received', message.requestId, request.commandType, { messageType: message.type, workerGeneration: generation });
      if (message.type === 'progress') {
        request.progress?.(message.payload);
        return;
      }
      pending.delete(message.requestId);
      if (message.type === 'generated-climate') recycleWhenIdle = true;
      if (message.type === 'error') {
        diagnostic('worker-error-message', message.requestId, request.commandType, { message: message.payload.message, workerGeneration: generation });
        request.reject(new Error(message.payload.message));
      } else {
        diagnostic('result-message-resolving', message.requestId, request.commandType, { messageType: message.type, workerGeneration: generation, pendingCountAfterDelete: pending.size });
        request.resolve(message.payload);
      }
      if (recycleWhenIdle && pending.size === 0 && !disposed) recycleWorker('climate-result-delivered');
    });
    next.addEventListener('error', event => {
      if (next !== worker || disposed) return;
      diagnostic('worker-error-event', undefined, undefined, { message: event.message, filename: event.filename, lineno: event.lineno, colno: event.colno, workerGeneration: generation });
      rejectAll(event.message || 'Planet Engine Worker failed.', 'worker-error-reject-all');
      recycleWorker('worker-error');
    });
    next.addEventListener('messageerror', event => {
      if (next !== worker || disposed) return;
      diagnostic('worker-messageerror-event', undefined, undefined, { dataType: typeof event.data, workerGeneration: generation });
      rejectAll('Planet Engine Worker result could not be deserialized on the main thread.', 'worker-messageerror-reject-all');
      recycleWorker('worker-messageerror');
    });
    return next;
  }

  function recycleWorker(reason: string): void {
    if (disposed) return;
    const previous = worker;
    diagnostic('worker-recycle-begin', undefined, undefined, { reason, pendingCount: pending.size, workerGeneration });
    const replacement = spawnWorker();
    worker = replacement;
    recycleWhenIdle = false;
    previous.terminate();
    diagnostic('worker-recycle-complete', undefined, undefined, { reason, workerGeneration });
  }

  worker = spawnWorker();

  function request<T extends WorldgenResult>(command: WorldgenRequestCommand, progress?: (progress: WorldgenGenerationProgress) => void): Promise<T> {
    if (disposed) return Promise.reject(new Error('Planet Engine client is disposed.'));
    return new Promise((resolve, reject) => {
      const commandType = command.type;
      pending.set(command.requestId, { resolve: result => resolve(result as T), reject, progress, commandType });
      diagnostic('request-post-begin', command.requestId, commandType, { pendingCount: pending.size, workerGeneration });
      try {
        worker.postMessage(command);
        diagnostic('request-post-complete', command.requestId, commandType, { workerGeneration });
      } catch (error) {
        pending.delete(command.requestId);
        diagnostic('request-post-threw', command.requestId, commandType, { message: error instanceof Error ? error.message : String(error), workerGeneration });
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  return {
    generateSynthetic(input) { validateSyntheticRequest(input); return request<WorldgenSyntheticResult>(worldgenSyntheticCommand(nextRequestId++, input)); },
    generateTopology(input) { validateTopologyRequest(input); return request<WorldgenTopologyResult>(worldgenTopologyCommand(nextRequestId++, input)); },
    generateTectonics(input) { validateTectonicsRequest(input); return request<WorldgenTectonicsResult>(worldgenTectonicsCommand(nextRequestId++, input)); },
    generateGeology(input) { validateGeologyRequest(input); return request<WorldgenGeologyResult>(worldgenGeologyCommand(nextRequestId++, input)); },
    generateLithosphere(input) { validateLithosphereRequest(input); return request<WorldgenLithosphereResult>(worldgenLithosphereCommand(nextRequestId++, input)); },
    generateInheritance(input) { validateInheritanceRequest(input); return request<WorldgenInheritanceResult>(worldgenInheritanceCommand(nextRequestId++, input)); },
    generateTopography(input) { validateTopographyRequest(input); return request<WorldgenTopographyResult>(worldgenTopographyCommand(nextRequestId++, input)); },
    generateClimate(input, onProgress) { validateClimateRequest(input); return request<WorldgenClimateResult>(worldgenClimateCommand(nextRequestId++, input), onProgress); },
    generateDrainage(input) { validateDrainageRequest(input); return request<WorldgenDrainageResult>(worldgenDrainageCommand(nextRequestId++, input)); },
    dispose() {
      if (disposed) return;
      disposed = true;
      diagnostic('client-dispose', undefined, undefined, { pendingCount: pending.size, workerGeneration });
      worker.terminate();
      rejectAll('Planet Engine client was disposed.', 'client-dispose-reject-all');
    },
  };
}
