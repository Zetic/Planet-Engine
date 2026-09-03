import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('GitHub Pages root serves the standalone Planet Engine Lab', () => {
  assert.ok(fs.existsSync('index.html'), 'Pages root requires index.html');
  assert.ok(fs.existsSync('styles/base.css'), 'Pages lab base styles must be committed');
  assert.ok(fs.existsSync('styles/worldgenLab.css'), 'Pages lab styles must be committed');

  const html = fs.readFileSync('index.html', 'utf8');
  assert.match(html, /PLANET ENGINE LAB/);
  assert.match(html, /dist\/worldgen\/diagnostics\/worldgenInheritanceLabStandalone\.js/);
  assert.match(html, /styles\/base\.css/);
  assert.match(html, /styles\/worldgenLab\.css/);
  assert.doesNotMatch(html, /Return to game/i);
  assert.doesNotMatch(html, /Legacy v7 gameplay worldgen/i);
});
