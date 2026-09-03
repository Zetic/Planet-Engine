import {
  WORLDGEN_PROTOCOL_VERSION,
  validateGeologyRequest,
  validateInheritanceRequest,
  validateLithosphereRequest,
  validateSyntheticRequest,
  validateTectonicsRequest,
  validateTopologyRequest,
  worldgenGeologyCommand,
  worldgenInheritanceCommand,
  worldgenLithosphereCommand,
  worldgenSyntheticCommand,
  worldgenTectonicsCommand,
  worldgenTopologyCommand,
  type WorldgenEvent,
  type WorldgenGeologyRequest,
  type WorldgenGeologyResult,
  type WorldgenInheritanceRequest,
  type WorldgenInheritanceResult,
  type WorldgenLithosphereRequest,
  type WorldgenLithosphereResult,
  type WorldgenSyntheticRequest,
  type WorldgenSyntheticResult,
  type WorldgenTectonicsRequest,
  type WorldgenTectonicsResult,
  type WorldgenTopologyRequest,
  type WorldgenTopologyResult,
} from './protocol.js';

type WorldgenResult = WorldgenSyntheticResult | WorldgenTopologyResult | WorldgenTectonicsResult | WorldgenGeologyResult | WorldgenLithosphereResult | WorldgenInheritanceResult;
type WorldgenRequestCommand = ReturnType<typeof worldgenSyntheticCommand> | ReturnType<typeof worldgenTopologyCommand> | ReturnType<typeof worldgenTectonicsCommand> | ReturnType<typeof worldgenGeologyCommand> | ReturnType<typeof worldgenLithosphereCommand> | ReturnType<typeof worldgenInheritanceCommand>;
interface PendingRequest { resolve: (result: WorldgenResult) => void; reject: (error: Error) => void; }

export interface WorldgenClient {
  generateSynthetic(request: WorldgenSyntheticRequest): Promise<WorldgenSyntheticResult>;
  generateTopology(request: WorldgenTopologyRequest): Promise<WorldgenTopologyResult>;
  generateTectonics(request: WorldgenTectonicsRequest): Promise<WorldgenTectonicsResult>;
  generateGeology(request: WorldgenGeologyRequest): Promise<WorldgenGeologyResult>;
  generateLithosphere(request: WorldgenLithosphereRequest): Promise<WorldgenLithosphereResult>;
  generateInheritance(request: WorldgenInheritanceRequest): Promise<WorldgenInheritanceResult>;
  dispose(): void;
}

export function createWorldgenClient(): WorldgenClient {
  const workerUrl = new URL('./worldgenWorker.js', import.meta.url);
  workerUrl.searchParams.set('v', String(WORLDGEN_PROTOCOL_VERSION));
  const worker = new Worker(workerUrl, { type: 'module' });
  const pending = new Map<number, PendingRequest>();
  let nextRequestId = 1;
  let disposed = false;

  function rejectAll(message: string): void { for (const request of pending.values()) request.reject(new Error(message)); pending.clear(); }
  worker.addEventListener('message', (event: MessageEvent<WorldgenEvent>) => {
    const message = event.data;
    if (!message || message.protocolVersion !== WORLDGEN_PROTOCOL_VERSION) return;
    const request = pending.get(message.requestId);
    if (!request) return;
    pending.delete(message.requestId);
    if (message.type === 'error') request.reject(new Error(message.payload.message)); else request.resolve(message.payload);
  });
  worker.addEventListener('error', event => rejectAll(event.message || 'Planet Engine Worker failed.'));

  function request<T extends WorldgenResult>(command: WorldgenRequestCommand): Promise<T> {
    if (disposed) return Promise.reject(new Error('Planet Engine client is disposed.'));
    return new Promise((resolve, reject) => { pending.set(command.requestId, { resolve: result => resolve(result as T), reject }); worker.postMessage(command); });
  }

  return {
    generateSynthetic(input) { validateSyntheticRequest(input); return request<WorldgenSyntheticResult>(worldgenSyntheticCommand(nextRequestId++, input)); },
    generateTopology(input) { validateTopologyRequest(input); return request<WorldgenTopologyResult>(worldgenTopologyCommand(nextRequestId++, input)); },
    generateTectonics(input) { validateTectonicsRequest(input); return request<WorldgenTectonicsResult>(worldgenTectonicsCommand(nextRequestId++, input)); },
    generateGeology(input) { validateGeologyRequest(input); return request<WorldgenGeologyResult>(worldgenGeologyCommand(nextRequestId++, input)); },
    generateLithosphere(input) { validateLithosphereRequest(input); return request<WorldgenLithosphereResult>(worldgenLithosphereCommand(nextRequestId++, input)); },
    generateInheritance(input) { validateInheritanceRequest(input); return request<WorldgenInheritanceResult>(worldgenInheritanceCommand(nextRequestId++, input)); },
    dispose() { if (disposed) return; disposed = true; worker.terminate(); rejectAll('Planet Engine client was disposed.'); },
  };
}
