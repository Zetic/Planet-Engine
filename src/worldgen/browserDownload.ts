export interface BrowserDownloadEnvironment {
  createObjectUrl(blob: Blob): string;
  revokeObjectUrl(url: string): void;
  createAnchor(): HTMLAnchorElement;
  appendAnchor(anchor: HTMLAnchorElement): void;
  scheduleCleanup(callback: () => void, delayMs: number): void;
}

const DOWNLOAD_URL_LIFETIME_MS = 1500;

function defaultBrowserDownloadEnvironment(): BrowserDownloadEnvironment {
  return {
    createObjectUrl: blob => URL.createObjectURL(blob),
    revokeObjectUrl: url => URL.revokeObjectURL(url),
    createAnchor: () => document.createElement('a'),
    appendAnchor: anchor => document.body.appendChild(anchor),
    scheduleCleanup: (callback, delayMs) => {
      window.setTimeout(callback, delayMs);
    },
  };
}

export function downloadTextFile(
  contents: string,
  filename: string,
  mimeType: string,
  environment: BrowserDownloadEnvironment = defaultBrowserDownloadEnvironment(),
): void {
  const blob = new Blob([contents], { type: mimeType });
  const url = environment.createObjectUrl(blob);
  const anchor = environment.createAnchor();
  anchor.href = url;
  anchor.download = filename;
  anchor.style.display = 'none';
  environment.appendAnchor(anchor);

  try {
    anchor.click();
  } catch (error) {
    anchor.remove();
    environment.revokeObjectUrl(url);
    throw error;
  }

  environment.scheduleCleanup(() => {
    anchor.remove();
    environment.revokeObjectUrl(url);
  }, DOWNLOAD_URL_LIFETIME_MS);
}
