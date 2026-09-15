import assert from 'node:assert/strict';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';
import { createWorldgenClient } from '../dist/worldgen/worldgenClient.js';

class FakeWorker {
  static instances: FakeWorker[] = [];
  readonly messages: unknown[] = [];
  terminated = false;
  private readonly messageListeners: Array<(event: MessageEvent) => void> = [];
  private readonly errorListeners: Array<(event: ErrorEvent) => void> = [];

  constructor(_url: URL, _options: WorkerOptions) {
    FakeWorker.instances.push(this);
  }

  addEventListener(type: string, listener: EventListenerOrEventListenerObject): void {
    const callback = typeof listener === 'function' ? listener : listener.handleEvent.bind(listener);
    if (type === 'message') this.messageListeners.push(callback as (event: MessageEvent) => void);
    if (type === 'error') this.errorListeners.push(callback as (event: ErrorEvent) => void);
  }

  postMessage(message: unknown): void { this.messages.push(message); }
  terminate(): void { this.terminated = true; }
  emitMessage(data: unknown): void {
    for (const listener of this.messageListeners) listener({ data } as MessageEvent);
  }
  emitError(message: string): void {
    for (const listener of this.errorListeners) listener({ message } as ErrorEvent);
  }
}

function installFakeWorker(): () => void {
  FakeWorker.instances = [];
  const original = globalThis.Worker;
  Object.defineProperty(globalThis, 'Worker', { configurable: true, writable: true, value: FakeWorker });
  return () => Object.defineProperty(globalThis, 'Worker', { configurable: true, writable: true, value: original });
}

test('cumulative climate generation recycles the grown WASM worker after transfer', async () => {
  const restore = installFakeWorker();
  try {
    const client = createWorldgenClient();
    const first = FakeWorker.instances[0]!;
    const climate = client.generateClimate({ seed: 'worker-recycle', coarseLevel: 2, fineLevel: 3, plateCount: 8 });
    first.emitMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: 1, type: 'generated-climate', payload: {} });
    await climate;
    assert.equal(first.terminated, true);
    assert.equal(FakeWorker.instances.length, 2);
    const replacement = FakeWorker.instances[1]!;
    assert.equal(replacement.terminated, false);
    client.dispose();
    assert.equal(replacement.terminated, true);
  } finally {
    restore();
  }
});

test('worker recycling waits until concurrent requests are settled', async () => {
  const restore = installFakeWorker();
  try {
    const client = createWorldgenClient();
    const first = FakeWorker.instances[0]!;
    const climate = client.generateClimate({ seed: 'worker-recycle-concurrent', coarseLevel: 2, fineLevel: 3, plateCount: 8 });
    const topology = client.generateTopology({ level: 2 });
    first.emitMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: 1, type: 'generated-climate', payload: {} });
    await climate;
    assert.equal(first.terminated, false);
    assert.equal(FakeWorker.instances.length, 1);
    first.emitMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: 2, type: 'generated-topology', payload: {} });
    await topology;
    assert.equal(first.terminated, true);
    assert.equal(FakeWorker.instances.length, 2);
    client.dispose();
  } finally {
    restore();
  }
});
