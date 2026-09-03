import { WORLDGEN_PROTOCOL_VERSION, validateClimateRequest, validateGeologyRequest, validateInheritanceRequest, validateLithosphereRequest, validateSyntheticRequest, validateTopographyRequest, validateTectonicsRequest, validateTopologyRequest, worldgenClimateCommand, worldgenGeologyCommand, worldgenInheritanceCommand, worldgenLithosphereCommand, worldgenSyntheticCommand, worldgenTopographyCommand, worldgenTectonicsCommand, worldgenTopologyCommand, } from './protocol.js';
export function createWorldgenClient() {
    const workerUrl = new URL('./worldgenWorker.js', import.meta.url);
    workerUrl.searchParams.set('v', String(WORLDGEN_PROTOCOL_VERSION));
    const worker = new Worker(workerUrl, { type: 'module' });
    const pending = new Map();
    let nextRequestId = 1;
    let disposed = false;
    function rejectAll(message) { for (const request of pending.values())
        request.reject(new Error(message)); pending.clear(); }
    worker.addEventListener('message', (event) => {
        const message = event.data;
        if (!message || message.protocolVersion !== WORLDGEN_PROTOCOL_VERSION)
            return;
        const request = pending.get(message.requestId);
        if (!request)
            return;
        pending.delete(message.requestId);
        if (message.type === 'error')
            request.reject(new Error(message.payload.message));
        else
            request.resolve(message.payload);
    });
    worker.addEventListener('error', event => rejectAll(event.message || 'Planet Engine Worker failed.'));
    function request(command) {
        if (disposed)
            return Promise.reject(new Error('Planet Engine client is disposed.'));
        return new Promise((resolve, reject) => { pending.set(command.requestId, { resolve: result => resolve(result), reject }); worker.postMessage(command); });
    }
    return {
        generateSynthetic(input) { validateSyntheticRequest(input); return request(worldgenSyntheticCommand(nextRequestId++, input)); },
        generateTopology(input) { validateTopologyRequest(input); return request(worldgenTopologyCommand(nextRequestId++, input)); },
        generateTectonics(input) { validateTectonicsRequest(input); return request(worldgenTectonicsCommand(nextRequestId++, input)); },
        generateGeology(input) { validateGeologyRequest(input); return request(worldgenGeologyCommand(nextRequestId++, input)); },
        generateLithosphere(input) { validateLithosphereRequest(input); return request(worldgenLithosphereCommand(nextRequestId++, input)); },
        generateInheritance(input) { validateInheritanceRequest(input); return request(worldgenInheritanceCommand(nextRequestId++, input)); },
        generateTopography(input) { validateTopographyRequest(input); return request(worldgenTopographyCommand(nextRequestId++, input)); },
        generateClimate(input) { validateClimateRequest(input); return request(worldgenClimateCommand(nextRequestId++, input)); },
        dispose() { if (disposed)
            return; disposed = true; worker.terminate(); rejectAll('Planet Engine client was disposed.'); },
    };
}
