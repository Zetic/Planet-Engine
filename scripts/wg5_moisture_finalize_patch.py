from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    s = p.read_text()
    if old not in s:
        raise SystemExit(f"missing anchor in {path}: {old[:100]!r}")
    p.write_text(s.replace(old, new, 1))


# Remove the old reconstructed moisture-cap diagnostic now that stage 4 exports exact runtime data.
p = Path("rust/interlink-worldgen/src/climate_calibration.rs")
s = p.read_text()
s = s.replace("    tangent_basis, ", "")
s = re.sub(
    r"\nfn edge_direction\(.*?\n\}\n\nfn symmetric_edge_normal_wind\(.*?\n\}\n",
    "\n",
    s,
    count=1,
    flags=re.S,
)
s = re.sub(
    r"\n    let mut east_bases = Vec::with_capacity\(count\);.*?    let phase_count = usize::from\(climate.metrics.orbital_phase_count\);\n    let phase_seconds = planet.orbital_period_s / phase_count as f64;",
    "\n    let phase_count = usize::from(climate.metrics.orbital_phase_count);",
    s,
    count=1,
    flags=re.S,
)
s = s.replace("    let mut moisture_cap_edges = 0_u64;\n", "")
s = s.replace("    let mut moisture_edges = 0_u64;\n", "")
s, removed = re.subn(
    r"\n        for a in 0\.\.count \{.*?\n        \}\n    \}\n\n    let rh_values",
    "\n    }\n\n    let rh_values",
    s,
    count=1,
    flags=re.S,
)
if removed != 1:
    raise SystemExit("obsolete reconstructed moisture loop was not removed")
# Adaptive patch already renamed the report field; ensure the initializer is sourced from runtime metrics.
s = re.sub(
    r"        moisture_transport_limiter_fraction: if moisture_edges > 0 \{.*?        \},\n",
    "        moisture_transport_limiter_fraction: climate.metrics.moisture_transport_limiter_fraction,\n        maximum_moisture_transport_substeps: climate.metrics.maximum_moisture_transport_substeps,\n",
    s,
    count=1,
    flags=re.S,
)
# If adaptive patch used the direct runtime initializer already, do not duplicate it.
s = s.replace(
    "        maximum_moisture_transport_substeps: climate.metrics.maximum_moisture_transport_substeps,\n        maximum_moisture_transport_substeps: climate.metrics.maximum_moisture_transport_substeps,\n",
    "        maximum_moisture_transport_substeps: climate.metrics.maximum_moisture_transport_substeps,\n",
)
p.write_text(s)

# Remove the superseded core helper if it is dead after the finite-volume edge implementation.
p = Path("rust/interlink-worldgen/src/climate.rs")
s = p.read_text()
s, _ = re.subn(
    r"\nfn symmetric_edge_normal_wind_m_s\(.*?\n\}\n(?=\nfn )",
    "\n",
    s,
    count=1,
    flags=re.S,
)
p.write_text(s)

# Permanent WASM surface and stage identity.
replace_once(
    "rust/interlink-worldgen-wasm/tests/climate_bridge.rs",
    "    assert_eq!(output.stage_version(), 3);",
    "    assert_eq!(output.stage_version(), 4);",
)
p = Path("rust/interlink-worldgen-wasm/src/climate_bridge.rs")
s = p.read_text()
anchor = '''    pub fn moisture_budget_relative_error(&self) -> f64 {\n        self.climate.metrics.moisture_budget_relative_error\n    }\n'''
insert = anchor + '''    pub fn moisture_transport_limiter_fraction(&self) -> f64 {\n        self.climate.metrics.moisture_transport_limiter_fraction\n    }\n    pub fn maximum_moisture_transport_substeps(&self) -> u8 {\n        self.climate.metrics.maximum_moisture_transport_substeps\n    }\n'''
if anchor not in s:
    raise SystemExit("WASM metric anchor missing")
p.write_text(s.replace(anchor, insert, 1))

# Permanent regression: the reduced latent-energy availability parameter must be causal.
p = Path("rust/interlink-worldgen/tests/climate_ensemble.rs")
s = p.read_text()
anchor = '''#[test]\nfn dry_planet_has_no_ocean_current_or_ocean_evaporation() {'''
test = '''#[test]\nfn latent_energy_availability_limits_ocean_evaporation_on_fixed_wg4_surface() {\n    let planet = PlanetPhysicalParameters::earthlike_reference();\n    let (topology, terrain) = generated_surface("wg5-latent-energy", planet);\n    let normal_request = ClimateRequest::new("wg5-latent-energy");\n    let normal = generate_coupled_climate(&topology, &terrain, planet, &normal_request).unwrap();\n\n    let mut constrained_request = normal_request.clone();\n    constrained_request.parameters.evaporation_energy_fraction = 0.05;\n    let constrained =\n        generate_coupled_climate(&topology, &terrain, planet, &constrained_request).unwrap();\n\n    assert!(constrained.metrics.global_evaporation_kg < normal.metrics.global_evaporation_kg);\n    assert!(constrained.metrics.moisture_budget_relative_error < 1.0e-8);\n    assert_ne!(constrained.metrics.climate_hash, normal.metrics.climate_hash);\n}\n\n'''
if anchor not in s:
    raise SystemExit("climate ensemble anchor missing")
p.write_text(s.replace(anchor, test + anchor, 1))

# Calibration regression follows runtime limiter instrumentation.
p = Path("rust/interlink-worldgen/tests/climate_calibration.rs")
s = p.read_text().replace(
    "first.reconstructed_moisture_edge_cap_fraction",
    "first.moisture_transport_limiter_fraction",
)
p.write_text(s)

# Stage-4 documentation.
p = Path("docs/worldgen-rewrite/WG5_CLIMATE.md")
s = p.read_text()
s = s.replace(
    "The thermally recalibrated climate algorithm is stage version `3`; version `3` rebuilds reduced shortwave forcing, atmospheric heat redistribution, and air-sea heat exchange while retaining the existing browser climate-state shape.",
    "The current climate algorithm is stage version `4`. Stage `3` rebuilt the reduced thermal budget; stage `4` rebuilds atmospheric moisture transport, evaporation, and convergence precipitation while preserving the accepted stage-3 thermal model.",
    1,
)
old = "Projected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps. Each undirected atmospheric interface uses a symmetric face-normal velocity reconstructed from both endpoint winds, so moisture routing is independent of which endpoint has the lower mesh sample index. Atmospheric terrain gradients are taken from the exposed land/sea surface: submerged bathymetry is not treated as an atmospheric obstacle or orographic precipitation source."
new = "Projected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Stage `4` gives atmospheric moisture its own conservative finite-volume graph transport. Seasonal-mean flow is integrated with adaptive Courant substeps, paired edge transfers conserve water mass, and a `1.0 m/s` climatological moisture-speed ceiling bounds unresolved travel distance without altering the physical wind field used by circulation, evaporation demand, or orography. The default minimum is four substeps, the adaptive ceiling is 64, and the donor CFL limit is `0.90`. Runtime limiter occupancy and maximum substeps are exported directly by the solver."
if old not in s:
    raise SystemExit("WG5 climate moisture paragraph missing")
s = s.replace(old, new, 1)
needle = "The reduced ocean-advection coupling is calibrated separately from the CFL safety cap."
paragraph = "Bulk ocean evaporation is now an aerodynamic mass-flux demand rather than a per-phase humidity relaxation. WG-5 remains a reduced climatology rather than a full surface-energy/GCM solve, so stage `4` applies a latent-energy availability ceiling: at most `0.45` of reduced absorbed ocean shortwave energy in a phase may support evaporation, using `2.45 MJ/kg` latent heat. This is a climatological availability bound, not a second thermal-energy sink; the independently calibrated stage-3 thermal EBM remains unchanged. Moisture convergence after each transport substep may precipitate sufficiently humid inflow before the existing condensation and orographic sinks.\n\n"
if needle not in s:
    raise SystemExit("WG5 climate insertion anchor missing")
s = s.replace(needle, paragraph + needle, 1)
p.write_text(s)

p = Path("docs/worldgen-rewrite/WG5_CALIBRATION.md")
s = p.read_text()
if "## Stage-4 moisture transport and precipitation recalibration" not in s:
    s += '''\n\n## Stage-4 moisture transport and precipitation recalibration\n\nStage `climate:coupled-surface@4` keeps the accepted stage-3 thermal solution fixed and replaces the resolution-cap-dominated moisture path. Selected reduced Earth-like defaults are bulk evaporation coefficient `0.0015`, latent-energy availability `0.45`, climatological moisture speed `1.0 m/s`, adaptive substeps `4–64`, donor CFL `0.90`, convergence RH threshold `0.60`, and convergence efficiency `0.35`.\n\nThe latent-energy fraction is a reduced-model availability ceiling, not a fully coupled surface-energy claim. Latent heat is not subtracted from the stage-3 temperature solve. Raw potential-evaporation demand remains diagnostic; PET/aridity recalibration is intentionally deferred.\n\nA fixed-ancestry resolution pair (`ci-wg5-l7`, coarse L5, 24 plates) changes only fine topology. The selected candidate measures `1125.604 mm/yr` precipitation and `5.743150e17 kg` evaporation at L6 versus `1122.388 mm/yr` and `5.726104e17 kg` at L7. Runtime limiter occupancy is `0.000000` / `0.000059`, maximum adaptive substeps are `33` / `64`, and moisture-budget errors are `1.72e-14` / `3.68e-14`. Global precipitation drift is about `0.3%`, replacing the pre-closure prototype's roughly `57%` L6→L7 increase while leaving the stage-3 thermal state unchanged.\n'''
p.write_text(s)

p = Path("docs/worldgen-rewrite/VALIDATION.md")
s = p.read_text()
if "## WG-5 coupled-climate gates" not in s:
    s += '''\n\n## WG-5 coupled-climate gates\n\nWG-5 requires deterministic identity, annual thermal convergence, atmospheric moisture conservation, finite sample-aligned climate fields, small projected ocean divergence, and permanent L7 execution. Stage `4` additionally requires runtime moisture donor-limiter occupancy below `1%` and same-coarse L6→L7 global precipitation drift below `10%` on seed `ci-wg5-l7`, coarse L5, 24 plates. This directly guards against recurrence of the former resolution-cap-dominated single-sweep moisture routing.\n'''
p.write_text(s)

# Permanent resolution acceptance script.
p = Path("scripts/check-wg5-moisture-resolution.sh")
p.write_text(r'''#!/usr/bin/env bash
set -euo pipefail
l6="$(mktemp)"; l7="$(mktemp)"
trap 'rm -f "$l6" "$l7"' EXIT
for fine in 6 7; do
  out="$l6"; [[ "$fine" == 7 ]] && out="$l7"
  cargo run --release -q -p interlink-worldgen-cli --example climate_calibration -- \
    --seed ci-wg5-l7 --coarse-level 5 --level "$fine" --plates 24 \
    --skip-orography-intervention | tee "$out"
done
python3 - "$l6" "$l7" <<'PY'
import re, sys

def parse(path):
    text=open(path).read()
    precip=float(re.search(r"hydrology_mm_year precip_mean=([0-9.eE+-]+)", text).group(1))
    m=re.search(r"transport wind_cap=[0-9.eE+-]+ moisture_limiter=([0-9.eE+-]+) moisture_max_substeps=([0-9]+)", text)
    error=float(re.search(r"water_budget_kg evaporation=[0-9.eE+-]+ precipitation=[0-9.eE+-]+ relative_error=([0-9.eE+-]+)", text).group(1))
    return precip,float(m.group(1)),int(m.group(2)),error
l6,l7=parse(sys.argv[1]),parse(sys.argv[2])
for label,v in (("L6",l6),("L7",l7)):
    assert 500 <= v[0] <= 2000, f"{label} precip={v[0]}"
    assert v[1] < 0.01, f"{label} limiter={v[1]}"
    assert 4 <= v[2] <= 64, f"{label} substeps={v[2]}"
    assert v[3] < 1e-8, f"{label} moisture error={v[3]}"
drift=abs(l7[0]-l6[0])/max(0.5*(l6[0]+l7[0]),1.0)
assert drift < 0.10, f"precip resolution drift={drift:.3%}"
print(f"WG-5 moisture resolution acceptance: L6={l6[0]:.3f} L7={l7[0]:.3f} drift={drift:.3%}")
PY
''')

p = Path("scripts/run-wg5-calibration.sh")
s = p.read_text()
s = s.replace(
    '  quality)\n    bash "$0" standard\n    run_case "ci-wg5-l7" 5 7 24\n    ;;',
    '  resolution)\n    bash scripts/check-wg5-moisture-resolution.sh\n    ;;\n  quality)\n    bash "$0" standard\n    bash scripts/check-wg5-moisture-resolution.sh\n    ;;',
    1,
)
s = s.replace('usage: $0 [smoke|standard|quality]', 'usage: $0 [smoke|standard|resolution|quality]')
p.write_text(s)
