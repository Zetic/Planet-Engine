import assert from 'node:assert/strict';
import test from 'node:test';
import { downloadTextFile, type BrowserDownloadEnvironment } from '../src/worldgen/browserDownload.js';

type FakeAnchor = HTMLAnchorElement & {
  clickCount: number;
  removed: boolean;
};

function makeAnchor(throwOnClick = false): FakeAnchor {
  const anchor = {
    href: '',
    download: '',
    style: { display: '' },
    clickCount: 0,
    removed: false,
    click() {
      this.clickCount += 1;
      if (throwOnClick) throw new Error('blocked download');
    },
    remove() {
      this.removed = true;
    },
  };
  return anchor as unknown as FakeAnchor;
}

test('browser download keeps the blob URL alive until deferred cleanup', () => {
  const anchor = makeAnchor();
  let appended = false;
  let revoked: string | null = null;
  let scheduledDelay = -1;
  let cleanup: (() => void) | null = null;
  let blobType = '';

  const environment: BrowserDownloadEnvironment = {
    createObjectUrl(blob) {
      blobType = blob.type;
      return 'blob:test-download';
    },
    revokeObjectUrl(url) {
      revoked = url;
    },
    createAnchor() {
      return anchor;
    },
    appendAnchor(value) {
      appended = value === anchor;
    },
    scheduleCleanup(callback, delayMs) {
      cleanup = callback;
      scheduledDelay = delayMs;
    },
  };

  downloadTextFile('{"ok":true}', 'planet-calibration-test.json', 'application/json;charset=utf-8', environment);

  assert.equal(appended, true);
  assert.equal(anchor.clickCount, 1);
  assert.equal(anchor.download, 'planet-calibration-test.json');
  assert.equal(anchor.href, 'blob:test-download');
  assert.equal(anchor.style.display, 'none');
  assert.equal(blobType, 'application/json;charset=utf-8');
  assert.equal(anchor.removed, false);
  assert.equal(revoked, null);
  assert.ok(scheduledDelay >= 1000, 'blob URL cleanup must be delayed long enough for navigation to consume it');
  assert.ok(cleanup);

  cleanup!();
  assert.equal(anchor.removed, true);
  assert.equal(revoked, 'blob:test-download');
});

test('browser download cleans up immediately when the synthetic click fails', () => {
  const anchor = makeAnchor(true);
  let revoked: string | null = null;
  let scheduled = false;
  const environment: BrowserDownloadEnvironment = {
    createObjectUrl() {
      return 'blob:failed-download';
    },
    revokeObjectUrl(url) {
      revoked = url;
    },
    createAnchor() {
      return anchor;
    },
    appendAnchor() {},
    scheduleCleanup() {
      scheduled = true;
    },
  };

  assert.throws(
    () => downloadTextFile('{}', 'failed.json', 'application/json;charset=utf-8', environment),
    /blocked download/,
  );
  assert.equal(anchor.removed, true);
  assert.equal(revoked, 'blob:failed-download');
  assert.equal(scheduled, false);
});
