import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
test('calibration packet stays compact, versioned, and available from the Pages lab', async () => {
  const packet = await readFile('src/worldgen/calibrationPacket.ts', 'utf8');
  const controller = await readFile('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const html = await readFile('index.html', 'utf8');
  assert.match(packet, /planet-engine-calibration@1/);
  assert.match(packet, /RANKED_LIMIT = 8/);
  assert.doesNotMatch(packet, /JSON\.stringify\(result/);
  assert.match(controller, /worldCalibrationMarkdown/);
  assert.match(controller, /worldCalibrationJson/);
  assert.match(controller, /worldgen-copy-calibration/);
  assert.match(controller, /worldgen-download-calibration/);
  assert.match(html, /Copy LLM Summary/);
  assert.match(html, /Download Calibration JSON/);
});
