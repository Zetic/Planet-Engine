from pathlib import Path

# Keep animated vector diagnostics bounded at high sample counts. The scalar
# background remains full quality on static redraws; animation uses the existing
# fast point path and redraws at 20 Hz rather than rebuilding 100k+ point paths
# at display refresh frequency.
path = Path('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts')
text = path.read_text()
needle = '''let animationRequest = 0;
let animationPhase = 0;

function orbitalPhase(): number { return Number(season.value) / 1000; }
'''
replacement = '''let animationRequest = 0;
let animationPhase = 0;
let lastVectorAnimationMs = Number.NEGATIVE_INFINITY;
const VECTOR_ANIMATION_INTERVAL_MS = 50;

function orbitalPhase(): number { return Number(season.value) / 1000; }
'''
assert needle in text
text = text.replace(needle, replacement, 1)
old = '''function vectorAnimationFrame(): void {
  animationRequest = 0;
  if (visualization.value === 'winds' || visualization.value === 'currents') {
    animationPhase = (animationPhase + 0.45) % 1000;
    redraw(false);
    animationRequest = requestAnimationFrame(vectorAnimationFrame);
  }
}
function updateAnimation(): void {
  if (animationRequest) { cancelAnimationFrame(animationRequest); animationRequest = 0; }
  if (visualization.value === 'winds' || visualization.value === 'currents') animationRequest = requestAnimationFrame(vectorAnimationFrame);
}
'''
new = '''function vectorAnimationFrame(timestampMs: number): void {
  animationRequest = 0;
  if (visualization.value === 'winds' || visualization.value === 'currents') {
    if (timestampMs - lastVectorAnimationMs >= VECTOR_ANIMATION_INTERVAL_MS) {
      animationPhase = (animationPhase + 0.9) % 1000;
      lastVectorAnimationMs = timestampMs;
      redraw(true);
    }
    animationRequest = requestAnimationFrame(vectorAnimationFrame);
  }
}
function updateAnimation(): void {
  if (animationRequest) { cancelAnimationFrame(animationRequest); animationRequest = 0; }
  lastVectorAnimationMs = Number.NEGATIVE_INFINITY;
  if (visualization.value === 'winds' || visualization.value === 'currents') animationRequest = requestAnimationFrame(vectorAnimationFrame);
}
'''
assert old in text
text = text.replace(old, new, 1)
path.write_text(text)

path = Path('tests/wg5Climate.test.ts')
text = path.read_text()
needle = '  assert.match(source, /requestAnimationFrame/);\n'
replacement = '''  assert.match(source, /requestAnimationFrame/);
  assert.match(source, /VECTOR_ANIMATION_INTERVAL_MS\s*=\s*50/);
  assert.match(source, /redraw\(true\)/);
'''
assert needle in text
text = text.replace(needle, replacement, 1)
path.write_text(text)

path = Path('tests/worldgenRewrite.test.ts')
text = path.read_text()
text = text.replace(
    "test('Planet Engine source stays independent from legacy gameplay world objects through WG-4', () => {",
    "test('Planet Engine source stays independent from legacy gameplay world objects through WG-5', () => {",
    1,
)
needle = "    'src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts',\n"
assert needle in text
text = text.replace(needle, needle + "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',\n", 1)
needle = "    'rust/interlink-worldgen/src/topography.rs',\n"
assert needle in text
text = text.replace(needle, needle + "    'rust/interlink-worldgen/src/climate.rs',\n    'rust/interlink-worldgen/tests/climate_ensemble.rs',\n", 1)
needle = "    'docs/worldgen-rewrite/TOPOGRAPHY.md',\n"
assert needle in text
text = text.replace(needle, needle + "    'docs/worldgen-rewrite/WG5_CLIMATE.md',\n", 1)
path.write_text(text)

path = Path('docs/worldgen-rewrite/WG5_CLIMATE.md')
text = path.read_text()
old = '''WG-5 intentionally includes a reduced B+ surface-ocean circulation model: currents are generated from wind stress, Coriolis response, WG-4 ocean connectivity, coastlines, and bathymetric mobility; SST transport feeds back into the atmospheric thermal solution. It does not attempt a full 3-D salinity/thermohaline ocean.
'''
new = '''WG-5 intentionally includes a reduced B+ surface-ocean circulation model. Wind stress produces candidate currents, latitude- and rotation-rate-dependent Coriolis response deflects them, WG-4 ocean connectivity removes land-crossing flow, and bathymetry reduces shallow-water mobility. The candidate field is converted to antisymmetric ocean-interface transports and passed through a deterministic graph pressure projection so the retained transport has a small divergence residual. ENU current vectors are reconstructed from those projected interface transports for diagnostics, while SST heat advection uses the projected transports directly; ocean diffusion also remains on ocean-only neighbors. SST then feeds back into atmospheric temperature and circulation. WG-5 does not attempt a full 3-D salinity/thermohaline ocean.

Rotation is a physical input rather than an Earth-fixed display assumption. Slower rotation broadens the reduced overturning/Hadley regime and weakens zonal/Coriolis control, while faster rotation narrows the overturning regime and increases rotational control. Coriolis deflection varies continuously with latitude and is exactly zero at the equator.
'''
assert old in text
text = text.replace(old, new, 1)
path.write_text(text)

path = Path('README.md')
text = path.read_text()
old = '''WG-5 derives deterministic seasonal insolation, temperature and pressure, prevailing winds, wind-driven surface-ocean circulation, sea-surface temperature and heat transport, atmospheric moisture, precipitation, aridity, and snow/sea-ice potential from the accepted WG-4 physical planet. Orbital phases are generation-time climatology samples; the Lab season slider reconstructs stored seasonal harmonics and does not run a live climate simulation.
'''
new = '''WG-5 derives deterministic seasonal insolation, temperature and pressure, rotation-sensitive prevailing winds, mass-projected wind-driven surface-ocean circulation, sea-surface temperature and conservative ocean heat transport, atmospheric moisture, precipitation, aridity, and snow/sea-ice potential from the accepted WG-4 physical planet. Orbital phases are generation-time climatology samples; the Lab season slider reconstructs stored seasonal harmonics and does not run a live climate simulation.
'''
assert old in text
text = text.replace(old, new, 1)
path.write_text(text)
