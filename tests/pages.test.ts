import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('GitHub Pages root serves the cumulative Planet Engine Lab through WG-5', () => {
  assert.ok(fs.existsSync('index.html'), 'Pages root requires index.html');
  assert.ok(fs.existsSync('worldgen-lab.html'), 'direct lab URL must remain available');
  assert.ok(fs.existsSync('styles/base.css'), 'Pages lab base styles must be committed');
  assert.ok(fs.existsSync('styles/worldgenLab.css'), 'Pages lab styles must be committed');

  const html = fs.readFileSync('index.html', 'utf8');
  const direct = fs.readFileSync('worldgen-lab.html', 'utf8');
  assert.equal(direct, html, 'Pages root and direct lab URL must expose identical controls');
  assert.match(html, /PLANET ENGINE LAB/);
  assert.match(html, /PLANET ENGINE · THROUGH WG-5/);
  assert.match(html, />Generate Planet</);
  assert.match(html, /id="worldgen-season"/);
  assert.match(html, /dist\/worldgen\/diagnostics\/worldgenClimateLabStandalone\.js/);
  assert.match(html, /styles\/base\.css/);
  assert.match(html, /styles\/worldgenLab\.css/);
  assert.doesNotMatch(html, /Return to game/i);
});
