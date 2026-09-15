import assert from 'node:assert/strict';
import test from 'node:test';
import { createWorldgenCrashRecorder, formatWorldgenCrashReport, summarizeTypedArrayPayload } from '../src/worldgen/worldgenCrashReport.ts';

test('worldgen crash report measures nested typed-array transport without retaining payload data', () => {
  const payload = { a: new Float32Array(4), nested: { b: new Uint8Array(3), c: new Uint32Array(2) } };
  const summary = summarizeTypedArrayPayload(payload);
  assert.equal(summary.arrayCount, 3);
  assert.equal(summary.totalBytes, 27);
  assert.equal(summary.largest[0]?.byteLength, 16);
  assert.equal(summary.largest[0]?.path, 'result.a');
});

test('worldgen crash recorder keeps request, progress checkpoint, and failure details', () => {
  const recorder = createWorldgenCrashRecorder(19);
  recorder.clear();
  recorder.startRun({ seed: 'debug-seed', coarseLevel: 5, fineLevel: 8, plateCount: 16 }, false);
  recorder.recordProgress({ stageId: 'transport-ready', stageIndex: 17, stageCount: 18, completed: 1, total: 1, elapsedMs: 1234, stageElapsedMs: 0, diagnostics: { transferBufferCount: 12, transferBytes: 4096 } });
  recorder.recordError(new Error('synthetic failure'), 'test');
  const report = recorder.snapshot();
  assert.equal(report.status, 'failed');
  assert.equal(report.request?.seed, 'debug-seed');
  assert.equal(report.lastCheckpoint, 'test:error');
  assert.match(report.failure?.message ?? '', /synthetic failure/);
  assert.ok(report.events.some(event => event.event === 'transport-ready'));
  assert.match(formatWorldgenCrashReport(report), /Planet Engine Worldgen Crash Report/);
  assert.match(formatWorldgenCrashReport(report), /synthetic failure/);
});
