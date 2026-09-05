import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('GitHub Pages root is the single cumulative Planet Engine Lab through WG-7C', () => {
  assert.ok(fs.existsSync('index.html'), 'Pages root requires index.html');
  assert.ok(!fs.existsSync('worldgen-lab.html'), 'secondary Lab HTML entrypoint must not exist');
  assert.ok(!fs.existsSync('drainage.html'), 'standalone drainage HTML entrypoint must not exist');
  assert.ok(fs.existsSync('styles/base.css'));
  assert.ok(fs.existsSync('styles/worldgenLab.css'));
  const html = fs.readFileSync('index.html', 'utf8');
  assert.match(html, /PLANET ENGINE LAB/);
  assert.match(html, /PLANET ENGINE · THROUGH WG-7C/);
  assert.match(html, /Effective erosive discharge/);
  assert.match(html, /Applied erosion depth/);
  assert.match(html, /Post-erosion potential discharge/);
  assert.match(html, /Potential annual discharge/);
  assert.match(html, /Contributing drainage area/);
  assert.match(html, /dist\/worldgen\/diagnostics\/worldgenClimateLabStandalone\.js/);
  assert.doesNotMatch(html, /Return to game/i);
});
