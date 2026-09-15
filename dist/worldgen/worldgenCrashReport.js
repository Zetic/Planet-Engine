const REPORT_VERSION = 1;
const STORAGE_KEY = 'planet-engine:last-worldgen-crash-report:v1';
const MAX_EVENTS = 512;
const MAX_LARGEST_ARRAYS = 24;
function nowIso() { return new Date().toISOString(); }
function nowMs() { return typeof performance !== 'undefined' ? performance.now() : Date.now(); }
function makeRunId() {
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function')
        return crypto.randomUUID();
    return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}
function finite(value) {
    return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}
function mib(bytes) { return Math.round(bytes / (1024 * 1024) * 1000) / 1000; }
function safeString(value) {
    try {
        return typeof value === 'string' ? value : JSON.stringify(value);
    }
    catch {
        return String(value);
    }
}
function collectWebGlEnvironment() {
    if (typeof document === 'undefined')
        return { available: false, reason: 'no-document' };
    try {
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl2');
        if (!gl)
            return { available: false, reason: 'webgl2-unavailable' };
        const debug = gl.getExtension('WEBGL_debug_renderer_info');
        return {
            available: true,
            version: gl.getParameter(gl.VERSION),
            shadingLanguageVersion: gl.getParameter(gl.SHADING_LANGUAGE_VERSION),
            vendor: gl.getParameter(gl.VENDOR),
            renderer: gl.getParameter(gl.RENDERER),
            unmaskedVendor: debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : undefined,
            unmaskedRenderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : undefined,
            maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE),
            maxRenderbufferSize: gl.getParameter(gl.MAX_RENDERBUFFER_SIZE),
            maxViewportDims: Array.from(gl.getParameter(gl.MAX_VIEWPORT_DIMS)),
            maxVertexAttribs: gl.getParameter(gl.MAX_VERTEX_ATTRIBS),
        };
    }
    catch (error) {
        return { available: false, reason: error instanceof Error ? error.message : String(error) };
    }
}
function collectEnvironment() {
    const navigatorLike = typeof navigator !== 'undefined'
        ? navigator
        : undefined;
    const memory = typeof performance !== 'undefined'
        ? performance.memory
        : undefined;
    return {
        page: typeof location !== 'undefined' ? `${location.origin}${location.pathname}` : undefined,
        userAgent: navigatorLike?.userAgent,
        platform: navigatorLike?.userAgentData?.platform ?? navigatorLike?.platform,
        mobile: navigatorLike?.userAgentData?.mobile,
        hardwareConcurrency: navigatorLike?.hardwareConcurrency,
        deviceMemoryGiB: navigatorLike?.deviceMemory,
        language: navigatorLike?.language,
        crossOriginIsolated: typeof crossOriginIsolated !== 'undefined' ? crossOriginIsolated : undefined,
        viewport: typeof window !== 'undefined' ? { width: window.innerWidth, height: window.innerHeight, devicePixelRatio: window.devicePixelRatio } : undefined,
        screen: typeof window !== 'undefined' ? { width: window.screen?.width, height: window.screen?.height, colorDepth: window.screen?.colorDepth } : undefined,
        visibilityState: typeof document !== 'undefined' ? document.visibilityState : undefined,
        jsHeap: memory ? {
            limitBytes: memory.jsHeapSizeLimit,
            totalBytes: memory.totalJSHeapSize,
            usedBytes: memory.usedJSHeapSize,
            limitMiB: mib(memory.jsHeapSizeLimit),
            totalMiB: mib(memory.totalJSHeapSize),
            usedMiB: mib(memory.usedJSHeapSize),
        } : undefined,
        webgl2: collectWebGlEnvironment(),
    };
}
function sanitizeDetails(details) {
    if (!details)
        return undefined;
    const sanitized = {};
    for (const [key, value] of Object.entries(details)) {
        if (value === undefined)
            continue;
        if (ArrayBuffer.isView(value)) {
            sanitized[key] = { constructor: value.constructor.name, length: value.byteLength / Math.max(1, value.BYTES_PER_ELEMENT ?? 1), byteLength: value.byteLength };
        }
        else if (value instanceof ArrayBuffer) {
            sanitized[key] = { constructor: 'ArrayBuffer', byteLength: value.byteLength };
        }
        else if (value instanceof Error) {
            sanitized[key] = { name: value.name, message: value.message, stack: value.stack };
        }
        else if (typeof value === 'bigint') {
            sanitized[key] = value.toString();
        }
        else {
            sanitized[key] = value;
        }
    }
    return sanitized;
}
export function summarizeTypedArrayPayload(value) {
    const arrays = [];
    const seen = new WeakSet();
    const visit = (candidate, path) => {
        if (ArrayBuffer.isView(candidate)) {
            const view = candidate;
            arrays.push({ path, constructor: candidate.constructor.name, length: view.length ?? view.byteLength, byteLength: view.byteLength });
            return;
        }
        if (candidate instanceof ArrayBuffer) {
            arrays.push({ path, constructor: 'ArrayBuffer', length: candidate.byteLength, byteLength: candidate.byteLength });
            return;
        }
        if (!candidate || typeof candidate !== 'object')
            return;
        if (seen.has(candidate))
            return;
        seen.add(candidate);
        if (Array.isArray(candidate)) {
            for (let index = 0; index < candidate.length; index += 1)
                visit(candidate[index], `${path}[${index}]`);
            return;
        }
        for (const [key, nested] of Object.entries(candidate))
            visit(nested, path ? `${path}.${key}` : key);
    };
    visit(value, 'result');
    const totalBytes = arrays.reduce((sum, entry) => sum + entry.byteLength, 0);
    arrays.sort((a, b) => b.byteLength - a.byteLength || a.path.localeCompare(b.path));
    return { arrayCount: arrays.length, totalBytes, totalMiB: mib(totalBytes), largest: arrays.slice(0, MAX_LARGEST_ARRAYS) };
}
function errorDetails(error, source) {
    if (error instanceof Error)
        return { source, name: error.name, message: error.message, stack: error.stack };
    return { source, name: 'NonError', message: safeString(error) };
}
function loadStored(protocolVersion) {
    if (typeof sessionStorage === 'undefined')
        return null;
    try {
        const raw = sessionStorage.getItem(STORAGE_KEY);
        if (!raw)
            return null;
        const parsed = JSON.parse(raw);
        if (parsed.reportVersion !== REPORT_VERSION || parsed.protocolVersion !== protocolVersion || !Array.isArray(parsed.events))
            return null;
        return parsed;
    }
    catch {
        return null;
    }
}
function persist(report) {
    if (typeof sessionStorage === 'undefined')
        return;
    try {
        sessionStorage.setItem(STORAGE_KEY, JSON.stringify(report));
    }
    catch { /* diagnostics must never break generation */ }
}
export function formatWorldgenCrashReport(report) {
    const request = report.request
        ? `${report.request.seed} · L${report.request.coarseLevel}→L${report.request.fineLevel} · ${report.request.plateCount} plates`
        : 'not started';
    const transport = report.transport ? `${report.transport.totalMiB.toFixed(1)} MiB across ${report.transport.arrayCount} typed arrays` : 'not received';
    const failure = report.failure ? `${report.failure.name}: ${report.failure.message}` : 'none recorded';
    const timeline = report.events.map(entry => {
        const detail = entry.details && Object.keys(entry.details).length > 0 ? ` ${JSON.stringify(entry.details)}` : '';
        return `- +${entry.elapsedMs.toFixed(1)} ms [${entry.source}] ${entry.event}${detail}`;
    }).join('\n');
    return [
        '# Planet Engine Worldgen Crash Report',
        '',
        `Report format: v${report.reportVersion}`,
        `Protocol: v${report.protocolVersion}`,
        `Run: ${report.runId}`,
        `Status: ${report.status}`,
        `Request: ${request}`,
        `Last checkpoint: ${report.lastCheckpoint ?? 'none'}`,
        `Transport payload: ${transport}`,
        `Failure: ${failure}`,
        '',
        '## Environment',
        '```json', JSON.stringify(report.environment, null, 2), '```',
        '',
        '## Timeline',
        timeline || '- no events recorded',
        '',
        '## Structured report',
        '```json', JSON.stringify(report, null, 2), '```',
    ].join('\n');
}
export function createWorldgenCrashRecorder(protocolVersion) {
    let runStartedMonotonic = nowMs();
    let sequence = 0;
    let report = loadStored(protocolVersion) ?? {
        reportVersion: REPORT_VERSION,
        protocolVersion,
        runId: makeRunId(),
        status: 'idle',
        environment: collectEnvironment(),
        events: [],
    };
    if (report.events.length > 0)
        sequence = Math.max(...report.events.map(event => event.sequence)) + 1;
    const save = () => persist(report);
    const record = (source, event, details) => {
        const entry = {
            sequence: sequence++,
            wallTimeIso: nowIso(),
            elapsedMs: Math.max(0, nowMs() - runStartedMonotonic),
            source,
            event,
            details: sanitizeDetails(details),
        };
        report.events.push(entry);
        if (report.events.length > MAX_EVENTS)
            report.events.splice(0, report.events.length - MAX_EVENTS);
        report.lastCheckpoint = `${source}:${event}`;
        report.environment = { ...report.environment, ...collectEnvironment() };
        save();
    };
    return {
        startRun(request, receiveOnly) {
            runStartedMonotonic = nowMs();
            sequence = 0;
            report = {
                reportVersion: REPORT_VERSION,
                protocolVersion,
                runId: makeRunId(),
                status: 'running',
                startedAtIso: nowIso(),
                request: { ...request },
                receiveOnly,
                environment: collectEnvironment(),
                events: [],
            };
            record('lab', 'run-start', { receiveOnly, seed: request.seed, coarseLevel: request.coarseLevel, fineLevel: request.fineLevel, plateCount: request.plateCount });
        },
        record,
        recordProgress(progress) {
            record('worker-progress', progress.stageId, {
                stageIndex: progress.stageIndex,
                stageCount: progress.stageCount,
                completed: progress.completed,
                total: progress.total,
                elapsedMs: finite(progress.elapsedMs),
                stageElapsedMs: finite(progress.stageElapsedMs),
                diagnostics: progress.diagnostics,
            });
        },
        recordResult(result) {
            const transport = summarizeTypedArrayPayload(result);
            report.transport = transport;
            report.result = {
                engineVersion: result.engineVersion,
                coarseLevel: result.coarseLevel,
                fineLevel: result.fineLevel,
                fineSampleCount: result.metrics.fineSampleCount,
                plateCount: result.metrics.plateCount,
                climateHash: result.metrics.climateHash,
                topographyHash: result.metrics.topographyHash,
                drainageHash: result.drainageMetrics.drainageHash,
                finalSurfaceHash: result.infillMetrics.postInfillSurfaceHash,
                stageDurationMs: result.stage.durationMs,
                generationTimings: result.generationTimings,
            };
            record('lab', 'result-payload-summarized', { arrayCount: transport.arrayCount, totalBytes: transport.totalBytes, totalMiB: transport.totalMiB });
            return transport;
        },
        recordError(error, source, details) {
            report.status = 'failed';
            report.endedAtIso = nowIso();
            report.failure = errorDetails(error, source);
            record(source, 'error', { ...details, name: report.failure.name, message: report.failure.message, stack: report.failure.stack });
            save();
        },
        complete(status = 'completed') {
            report.status = status;
            report.endedAtIso = nowIso();
            record('lab', status, {});
            save();
        },
        snapshot() {
            return JSON.parse(JSON.stringify({ ...report, environment: { ...report.environment, ...collectEnvironment() } }));
        },
        toJson() { return JSON.stringify(this.snapshot(), null, 2); },
        toMarkdown() { return formatWorldgenCrashReport(this.snapshot()); },
        hasAttempt() { return report.status !== 'idle' || report.events.length > 0; },
        clear() {
            sequence = 0;
            report = { reportVersion: REPORT_VERSION, protocolVersion, runId: makeRunId(), status: 'idle', environment: collectEnvironment(), events: [] };
            if (typeof sessionStorage !== 'undefined') {
                try {
                    sessionStorage.removeItem(STORAGE_KEY);
                }
                catch { /* ignore */ }
            }
        },
    };
}
export function installWorldgenGlobalFailureCapture(recorder) {
    if (typeof window === 'undefined')
        return () => undefined;
    const onError = (event) => recorder.recordError(event.error ?? event.message, 'window-error', { filename: event.filename, lineno: event.lineno, colno: event.colno });
    const onRejection = (event) => recorder.recordError(event.reason, 'unhandled-rejection');
    const onVisibility = () => recorder.record('browser', 'visibility-change', { visibilityState: document.visibilityState });
    window.addEventListener('error', onError);
    window.addEventListener('unhandledrejection', onRejection);
    document.addEventListener('visibilitychange', onVisibility);
    return () => {
        window.removeEventListener('error', onError);
        window.removeEventListener('unhandledrejection', onRejection);
        document.removeEventListener('visibilitychange', onVisibility);
    };
}
