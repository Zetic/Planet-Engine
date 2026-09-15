from pathlib import Path

p = Path('src/worldgen/worldgenCrashReport.ts')
text = p.read_text()

old = """function collectEnvironment(): Record<string, unknown> {
  const navigatorLike = typeof navigator !== 'undefined'
    ? navigator as Navigator & { deviceMemory?: number; userAgentData?: { platform?: string; mobile?: boolean } }
    : undefined;
  const memory = typeof performance !== 'undefined'
    ? (performance as Performance & { memory?: { jsHeapSizeLimit: number; totalJSHeapSize: number; usedJSHeapSize: number } }).memory
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
"""
new = """function collectDynamicEnvironment(): Record<string, unknown> {
  const memory = typeof performance !== 'undefined'
    ? (performance as Performance & { memory?: { jsHeapSizeLimit: number; totalJSHeapSize: number; usedJSHeapSize: number } }).memory
    : undefined;
  return {
    visibilityState: typeof document !== 'undefined' ? document.visibilityState : undefined,
    jsHeap: memory ? {
      limitBytes: memory.jsHeapSizeLimit,
      totalBytes: memory.totalJSHeapSize,
      usedBytes: memory.usedJSHeapSize,
      limitMiB: mib(memory.jsHeapSizeLimit),
      totalMiB: mib(memory.totalJSHeapSize),
      usedMiB: mib(memory.usedJSHeapSize),
    } : undefined,
  };
}

function collectEnvironment(): Record<string, unknown> {
  const navigatorLike = typeof navigator !== 'undefined'
    ? navigator as Navigator & { deviceMemory?: number; userAgentData?: { platform?: string; mobile?: boolean } }
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
    ...collectDynamicEnvironment(),
    webgl2: collectWebGlEnvironment(),
  };
}
"""
if new not in text:
    if old not in text:
        raise SystemExit('collectEnvironment anchor not found')
    text = text.replace(old, new, 1)

text = text.replace(
    "report.environment = { ...report.environment, ...collectEnvironment() };",
    "report.environment = { ...report.environment, ...collectDynamicEnvironment() };",
)
text = text.replace(
    "return JSON.parse(JSON.stringify({ ...report, environment: { ...report.environment, ...collectEnvironment() } })) as WorldgenCrashReportSnapshot;",
    "return JSON.parse(JSON.stringify({ ...report, environment: { ...report.environment, ...collectDynamicEnvironment() } })) as WorldgenCrashReportSnapshot;",
)
p.write_text(text)
print('made crash recorder environment sampling lightweight')
