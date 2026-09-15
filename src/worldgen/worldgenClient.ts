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
interface PendingRequest { resolve: (result: WorldgenResult) => void; reject: (error: Error) => void; progress?: (progress: WorldgenGenerationProgress) => void; }

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

export function createWorldgenClient(): WorldgenClient {
  const workerUrl = new URL('./worldgenWorker.js', import.meta.url);
  workerUrl.searchParams.set('v', String(WORLDGEN_PROTOCOL_VERSION));
  const pending = new Map<number, PendingRequest>();
  let nextRequestId = 1;
  let disposed = false;
  let recycleWhenIdle = false;
  let worker: Worker;

  function rejectAll(message: string): void {
    for (const request of pending.values()) request.reject(new Error(message));
    pending.clear();
  }

  function spawnWorker(): Worker {
    const next = new Worker(workerUrl, { type: 'module' });
    next.addEventListener('message', (event: MessageEvent<WorldgenEvent>) => {
      if (next !== worker) return;
      const message = event.data;
      if (!message || message.protocolVersion !== WORLDGEN_PROTOCOL_VERSION) return;
      const request = pending.get(message.requestId);
      if (!request) return;
      if (message.type === 'progress') {
        request.progress?.(message.payload);
        return;
      }
      pending.delete(message.requestId);
      if (message.type === 'generated-climate') recycleWhenIdle = true;
      if (message.type === 'error') request.reject(new Error(message.payload.message));
      else request.resolve(message.payload);
      if (recycleWhenIdle && pending.size === 0 && !disposed) recycleWorker();
    });
    next.addEventListener('error', event => {
      if (next !== worker || disposed) return;
      rejectAll(event.message || 'Planet Engine Worker failed.');
      recycleWorker();
    });
    return next;
  }

  function recycleWorker(): void {
    if (disposed) return;
    const previous = worker;
    const replacement = spawnWorker();
    worker = replacement;
    recycleWhenIdle = false;
    previous.terminate();
  }

  worker = spawnWorker();

  function request<T extends WorldgenResult>(command: WorldgenRequestCommand, progress?: (progress: WorldgenGenerationProgress) => void): Promise<T> {
    if (disposed) return Promise.reject(new Error('Planet Engine client is disposed.'));
    return new Promise((resolve, reject) => {
      pending.set(command.requestId, { resolve: result => resolve(result as T), reject, progress });
      worker.postMessage(command);
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
      worker.terminate();
      rejectAll('Planet Engine client was disposed.');
    },
  };
}
