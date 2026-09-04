export const WORLDGEN_PROTOCOL_VERSION = 14;
export const WORLDGEN_SYNTHETIC_MAX_SAMPLES = 4_194_304;
export const WORLDGEN_TOPOLOGY_MAX_LEVEL = 7;
export const WORLDGEN_TECTONICS_MAX_LEVEL = 6;
export const WORLDGEN_GEOLOGY_MAX_LEVEL = 6;
export const WORLDGEN_LITHOSPHERE_MAX_LEVEL = 6;
export const WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_INHERITANCE_FINE_MAX_LEVEL = 7;
export const WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL = 7;
export const WORLDGEN_CLIMATE_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_CLIMATE_FINE_MAX_LEVEL = 7;
export const WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_DRAINAGE_FINE_MAX_LEVEL = 7;
export const WORLDGEN_INVALID_SAMPLE_ID = 0xffff_ffff;
export const WORLDGEN_DRAINAGE_OUTLET_NONE = 0;
export const WORLDGEN_DRAINAGE_OUTLET_OCEAN = 1;
export const WORLDGEN_DRAINAGE_OUTLET_INTERNAL = 2;
export const WORLDGEN_TECTONICS_MIN_PLATES = 4;
export const WORLDGEN_TECTONICS_MAX_PLATES = 48;
export const WORLDGEN_BOUNDARY_CONVERGENT = 1;
export const WORLDGEN_BOUNDARY_DIVERGENT = 2;
export const WORLDGEN_BOUNDARY_TRANSFORM = 3;
export const WORLDGEN_CRUST_OCEANIC = 1;
export const WORLDGEN_CRUST_TRANSITIONAL = 2;
export const WORLDGEN_CRUST_CONTINENTAL = 3;
export const WORLDGEN_PLATE_MAJOR = 1;
export const WORLDGEN_PLATE_INTERMEDIATE = 2;
export const WORLDGEN_PLATE_MINOR = 3;
export const WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION = 1;
export const WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION = 2;
export const WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION = 3;
export const WORLDGEN_GEOLOGY_OCEANIC_RIDGE = 4;
export const WORLDGEN_GEOLOGY_CONTINENTAL_RIFT = 5;
export const WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE = 6;
export const WORLDGEN_GEOLOGY_TRANSFORM = 7;
export const WORLDGEN_SUBDUCTION_NONE = 0;
export const WORLDGEN_SUBDUCTION_PLATE_A = 1;
export const WORLDGEN_SUBDUCTION_PLATE_B = 2;
export const WORLDGEN_STRUCTURE_NONE = 0;
export const WORLDGEN_STRUCTURE_SUTURE = 1;
export const WORLDGEN_STRUCTURE_RIFT = 2;
export const WORLDGEN_STRUCTURE_TRANSFORM = 3;
export const WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN = 4;
export const WORLDGEN_FRAGMENT_TERRANE = 1;
export const WORLDGEN_FRAGMENT_MICROPLATE = 2;
export function validateSyntheticRequest(request) {
    if (!request.seed.trim())
        throw new Error('Worldgen seed must not be empty.');
    for (const [name, value] of [['width', request.width], ['height', request.height]])
        if (!Number.isInteger(value) || value <= 0)
            throw new Error(`Worldgen ${name} must be a positive integer.`);
    const samples = request.width * request.height;
    if (!Number.isSafeInteger(samples) || samples > WORLDGEN_SYNTHETIC_MAX_SAMPLES)
        throw new Error(`WG-0 synthetic diagnostics are limited to ${WORLDGEN_SYNTHETIC_MAX_SAMPLES.toLocaleString()} samples.`);
}
export function validateTopologyRequest(request) {
    if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_TOPOLOGY_MAX_LEVEL)
        throw new Error(`WG-1 browser topology level must be an integer from 0 through ${WORLDGEN_TOPOLOGY_MAX_LEVEL}.`);
}
export function validateTectonicsRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-2 tectonic seed must not be empty.');
    if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_TECTONICS_MAX_LEVEL)
        throw new Error(`WG-2 browser tectonics level must be an integer from 0 through ${WORLDGEN_TECTONICS_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-2 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const samples = 10 * (4 ** request.level) + 2;
    if (request.plateCount > samples)
        throw new Error('WG-2 plate count cannot exceed topology sample count.');
}
export function validateGeologyRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-3 geology seed must not be empty.');
    if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_GEOLOGY_MAX_LEVEL)
        throw new Error(`WG-3 browser geology level must be an integer from 0 through ${WORLDGEN_GEOLOGY_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-3 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const samples = 10 * (4 ** request.level) + 2;
    if (request.plateCount > samples)
        throw new Error('WG-3 plate count cannot exceed topology sample count.');
}
export function validateLithosphereRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-3.5 lithosphere seed must not be empty.');
    if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_LITHOSPHERE_MAX_LEVEL)
        throw new Error(`WG-3.5 browser lithosphere level must be an integer from 0 through ${WORLDGEN_LITHOSPHERE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-3.5 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const samples = 10 * (4 ** request.level) + 2;
    if (request.plateCount > samples)
        throw new Error('WG-3.5 plate count cannot exceed topology sample count.');
}
export function validateInheritanceRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-3.75 inheritance seed must not be empty.');
    if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL)
        throw new Error(`WG-3.75 coarse level must be an integer from 0 through ${WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_INHERITANCE_FINE_MAX_LEVEL)
        throw new Error(`WG-3.75 fine level must be an integer from coarse level through ${WORLDGEN_INHERITANCE_FINE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-3.75 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
    if (request.plateCount > coarseSamples)
        throw new Error('WG-3.75 plate count cannot exceed coarse topology sample count.');
}
export function validateTopographyRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-4 topography seed must not be empty.');
    if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL)
        throw new Error(`WG-4 coarse level must be an integer from 0 through ${WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL)
        throw new Error(`WG-4 fine level must be an integer from coarse level through ${WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-4 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
    if (request.plateCount > coarseSamples)
        throw new Error('WG-4 plate count cannot exceed coarse topology sample count.');
}
export function validateClimateRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-5 climate seed must not be empty.');
    if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_CLIMATE_COARSE_MAX_LEVEL)
        throw new Error(`WG-5 coarse level must be an integer from 0 through ${WORLDGEN_CLIMATE_COARSE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_CLIMATE_FINE_MAX_LEVEL)
        throw new Error(`WG-5 fine level must be an integer from coarse level through ${WORLDGEN_CLIMATE_FINE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-5 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
    if (request.plateCount > coarseSamples)
        throw new Error('WG-5 plate count cannot exceed coarse topology sample count.');
}
export function validateDrainageRequest(request) {
    if (!request.seed.trim())
        throw new Error('WG-6A drainage seed must not be empty.');
    if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL)
        throw new Error(`WG-6A coarse level must be an integer from 0 through ${WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_DRAINAGE_FINE_MAX_LEVEL)
        throw new Error(`WG-6A fine level must be an integer from coarse level through ${WORLDGEN_DRAINAGE_FINE_MAX_LEVEL}.`);
    if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES)
        throw new Error(`WG-6A plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
    const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
    if (request.plateCount > coarseSamples)
        throw new Error('WG-6A plate count cannot exceed coarse topology sample count.');
}
export function worldgenSyntheticCommand(requestId, payload) { validateSyntheticRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-synthetic', payload }; }
export function worldgenTopologyCommand(requestId, payload) { validateTopologyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-topology', payload }; }
export function worldgenTectonicsCommand(requestId, payload) { validateTectonicsRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-tectonics', payload }; }
export function worldgenGeologyCommand(requestId, payload) { validateGeologyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-geology', payload }; }
export function worldgenLithosphereCommand(requestId, payload) { validateLithosphereRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-lithosphere', payload }; }
export function worldgenInheritanceCommand(requestId, payload) { validateInheritanceRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-inheritance', payload }; }
export function worldgenTopographyCommand(requestId, payload) { validateTopographyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-topography', payload }; }
export function worldgenClimateCommand(requestId, payload) { validateClimateRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-climate', payload }; }
export function worldgenDrainageCommand(requestId, payload) { validateDrainageRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-drainage', payload }; }
