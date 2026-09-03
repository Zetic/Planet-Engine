from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    s = p.read_text()
    if old not in s:
        raise SystemExit(f"missing anchor in {path}: {old[:80]!r}")
    p.write_text(s.replace(old, new, 1))


# Remove obsolete reconstructed moisture-cap work now that the solver exports runtime metrics.
p = Path("rust/interlink-worldgen/src/climate_calibration.rs")
s = p.read_text()
s = s.replace("reconstructed_moisture_edge_cap_fraction", "moisture_transport_limiter_fraction")
# The adaptive prototype leaves its old reconstructed counters unused; remove the whole old counter
# section when the familiar declaration/finalizer anchors are present.
start = s.find("    let mut moisture_cap_edges = 0_u64;\n")
if start >= 0:
    end_marker = "    let mean_state_relative_humidity_p05 ="
    end = s.find(end_marker, start)
    if end < 0:
        raise SystemExit("could not find end of obsolete moisture reconstruction block")
    # Preserve wind-cap reconstruction before the moisture declarations when present; only remove
    # from the obsolete moisture counters through the next mean-state diagnostic.
    s = s[:start] + s[end:]
# If a residual initializer still points at a reconstructed local, make runtime instrumentation explicit.
s = s.replace(
    "        moisture_transport_limiter_fraction,\n",
    "        moisture_transport_limiter_fraction: climate.metrics.moisture_transport_limiter_fraction,\n",
    1,
) if "        moisture_transport_limiter_fraction,\n" in s else s
p.write_text(s)

# Remove the superseded standalone symmetric wind helper from core if it became dead after the
# finite-volume edge implementation. Do not touch the new edge-local symmetric calculation.
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

# Permanent WASM surface: stage identity plus exact runtime transport health metrics.
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
    raise SystemExit("WASM moisture metric anchor missing")
p.write_text(s.replace(anchor, insert, 1))

# Causal regression for the reduced latent-energy availability ceiling.
p = Path("rust/interlink-worldgen/tests/climate_ensemble.rs")
s = p.read_text()
anchor = '''#[test]\nfn dry_planet_has_no_ocean_current_or_ocean_evaporation() {'''
test = '''#[test]\nfn latent_energy_availability_limits_ocean_evaporation_on_fixed_wg4_surface() {\n    let planet = PlanetPhysicalParameters::earthlike_reference();\n    let (topology, terrain) = generated_surface("wg5-latent-energy", planet);\n    let normal_request = ClimateRequest::new("wg5-latent-energy");\n    let normal = generate_coupled_climate(&topology, &terrain, planet, &normal_request).unwrap();\n\n    let mut constrained_request = normal_request.clone();\n    constrained_request.parameters.evaporation_energy_fraction = 0.05;\n    let constrained =\n        generate_coupled_climate(&topology, &terrain, planet, &constrained_request).unwrap();\n\n    assert!(\n        constrained.metrics.global_evaporation_kg < normal.metrics.global_evaporation_kg,\n        "lower latent-energy availability must causally reduce ocean evaporation",\n    );\n    assert!(constrained.metrics.moisture_budget_relative_error < 1.0e-8);\n    assert_ne!(constrained.metrics.climate_hash, normal.metrics.climate_hash);\n}\n\n'''
if anchor not in s:
    raise SystemExit("climate ensemble insertion anchor missing")
p.write_text(s.replace(anchor, test + anchor, 1))

# Calibration test follows runtime transport instrumentation instead of reconstructed edge caps.
p = Path("rust/interlink-worldgen/tests/climate_calibration.rs")
s = p.read_text().replace(
    "first.reconstructed_moisture_edge_cap_fraction",
    "first.moisture_transport_limiter_fraction",
)
p.write_text(s)

# Stage-4 documentation. Preserve stage-3 thermal history while describing the new reduced moisture solve.
p = Path("docs/worldgen-rewrite/WG5_CLIMATE.md")
s = p.read_text()
s = s.replace(
    "The thermally recalibrated climate algorithm is stage version `3`; version `3` rebuilds reduced shortwave forcing, atmospheric heat redistribution, and air-sea heat exchange while retaining the existing browser climate-state shape.",
    "The current climate algorithm is stage version `4`. Stage `3` rebuilt the reduced thermal budget; stage `4` rebuilds atmospheric moisture transport, evaporation, and convergence precipitation while preserving the accepted stage-3 thermal model.",
    1,
)
old = "Projected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps. Each undirected atmospheric interface uses a symmetric face-normal velocity reconstructed from both endpoint winds, so moisture routing is independent of which endpoint has the lower mesh sample index. Atmospheric terrain gradients are taken from the exposed land/sea surface: submerged bathymetry is not treated as an atmospheric obstacle or orographic precipitation source."
new = "Projected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Stage `4` gives atmospheric moisture its own conservative finite-volume graph transport. A seasonal wind state is integrated with adaptive Courant substeps, paired edge transfers conserve water mass, and a `1.0 m/s` climatological transport-speed ceiling limits how far the reduced seasonal-mean flow may advect humidity without altering the physical wind field used by circulation, evaporation demand, or orography. The default minimum is four substeps, the adaptive ceiling is 64, and the donor CFL limit is `0.90`. Runtime limiter occupancy and maximum substeps are exported as solver diagnostics rather than reconstructed from compressed annual harmonics."
if old not in s:
    raise SystemExit("WG5 climate moisture paragraph anchor missing")
s = s.replace(old, new, 1)
needle = "The reduced ocean-advection coupling is calibrated separately from the CFL safety cap."
paragraph = "Bulk ocean evaporation is now expressed as an aerodynamic mass-flux demand rather than a per-phase humidity relaxation. Because WG-5 intentionally remains a reduced climatology rather than a full surface-energy/GCM solve, stage `4` also applies a latent-energy availability ceiling: at most `0.45` of the reduced absorbed ocean shortwave energy in an orbital phase may support evaporation, using `2.45 MJ/kg` latent heat. This is a climatological availability bound, not a second thermal-energy sink; the independently calibrated stage-3 thermal EBM remains unchanged. Moisture convergence after each conservative transport substep can precipitate sufficiently humid inflow before the existing condensation and orographic sinks are applied.\n\n"
if needle not in s:
    raise SystemExit("WG5 climate ocean coupling anchor missing")
s = s.replace(needle, paragraph + needle, 1)
p.write_text(s)

p = Path("docs/worldgen-rewrite/WG5_CALIBRATION.md")
s = p.read_text()
append = '''\n\n## Stage-4 moisture transport and precipitation recalibration\n\nStage `climate:coupled-surface@4` keeps the accepted stage-3 thermal solution fixed and replaces the resolution-cap-dominated moisture source/transport path. The selected reduced Earth-like defaults are:\n\n- bulk aerodynamic evaporation coefficient: `0.0015`;\n- reduced latent-energy availability fraction: `0.45`;\n- climatological moisture transport speed ceiling: `1.0 m/s`;\n- minimum adaptive transport substeps: `4`;\n- maximum adaptive transport substeps: `64`;\n- donor CFL limit: `0.90`;\n- convergence-precipitation RH threshold: `0.60`;\n- convergence-precipitation efficiency: `0.35`.\n\nThe latent-energy fraction is deliberately a reduced-model availability ceiling, not a claim of a fully coupled surface energy budget. WG-5 does not subtract latent heat from the stage-3 temperature solve; instead it prevents bulk aerodynamic moisture demand from implying a planetary evaporation cycle unsupported by the reduced absorbed-energy scale. Raw potential-evaporation demand remains diagnostic and PET/aridity recalibration is deferred to the next WG-5 follow-up.\n\nThe permanent resolution acceptance holds tectonic ancestry fixed (`ci-wg5-l7`, coarse L5, 24 plates) and changes only fine topology L6→L7. With the selected defaults the measured pair is:\n\n| fine level | mean precip | global evaporation | limiter fraction | max substeps | moisture error |\n| --- | ---: | ---: | ---: | ---: | ---: |\n| L6 | 1125.604 mm/yr | 5.743150e17 kg | 0.000000 | 33 | 1.72e-14 |\n| L7 | 1122.388 mm/yr | 5.726104e17 kg | 0.000059 | 64 | 3.68e-14 |\n\nMean precipitation differs by about `0.3%`, replacing the pre-closure prototype's roughly `57%` L6→L7 increase. The stage-3 temperature/SST state is unchanged by this moisture calibration. Permanent acceptance therefore guards both exact moisture conservation and resolution stability rather than tuning one visual seed.\n'''
if "## Stage-4 moisture transport and precipitation recalibration" not in s:
    s += append
p.write_text(s)

p = Path("docs/worldgen-rewrite/VALIDATION.md")
s = p.read_text()
append = '''\n\n## WG-5 coupled-climate gates\n\nWG-5 acceptance requires deterministic stage identity; annual thermal convergence within the configured spin-up bound; exact atmospheric moisture conservation within tolerance; finite sample-aligned thermal, wind, current, humidity and precipitation fields; small projected ocean-transport divergence; and permanent L7 execution. Stage `4` additionally requires runtime moisture donor-limiter occupancy below `1%` on the fixed quality pair and a same-coarse L6→L7 global-precipitation difference below `10%`. The fixed pair uses seed `ci-wg5-l7`, coarse L5 and 24 plates so only the fine physical mesh changes. This gate exists specifically to prevent recurrence of the former resolution-cap-dominated single-sweep moisture routing.\n'''
if "## WG-5 coupled-climate gates" not in s:
    s += append
p.write_text(s)

# Permanent fixed-coarse acceptance script.
p = Path("scripts/check-wg5-moisture-resolution.sh")
p.write_text(r'''#!/usr/bin/env bash
set -euo pipefail

l6="$(mktemp)"
l7="$(mktemp)"
trap 'rm -f "$l6" "$l7"' EXIT

run_case() {
  local fine="$1"
  local out="$2"
  cargo run --release -q -p interlink-worldgen-cli --example climate_calibration -- \
    --seed ci-wg5-l7 --coarse-level 5 --level "$fine" --plates 24 \
    --skip-orography-intervention | tee "$out"
}

run_case 6 "$l6"
run_case 7 "$l7"

python3 - "$l6" "$l7" <<'PY'
import re
import sys


def parse(path):
    text = open(path).read()
    precip = float(re.search(r"hydrology_mm_year precip_mean=([0-9.eE+-]+)", text).group(1))
    m = re.search(r"transport wind_cap=[0-9.eE+-]+ moisture_limiter=([0-9.eE+-]+) moisture_max_substeps=([0-9]+)", text)
    limiter = float(m.group(1))
    substeps = int(m.group(2))
    error = float(re.search(r"water_budget_kg evaporation=[0-9.eE+-]+ precipitation=[0-9.eE+-]+ relative_error=([0-9.eE+-]+)", text).group(1))
    return precip, limiter, substeps, error

l6 = parse(sys.argv[1])
l7 = parse(sys.argv[2])
for label, values in (("L6", l6), ("L7", l7)):
    precip, limiter, substeps, error = values
    assert 500.0 <= precip <= 2000.0, f"{label} global precip outside broad Earth-like gate: {precip}"
    assert limiter < 0.01, f"{label} moisture limiter occupancy too high: {limiter}"
    assert error < 1.0e-8, f"{label} moisture budget failed: {error}"
    assert 4 <= substeps <= 64, f"{label} adaptive substep diagnostic invalid: {substeps}"
relative = abs(l7[0] - l6[0]) / max(0.5 * (l6[0] + l7[0]), 1.0)
assert relative < 0.10, f"L6/L7 precipitation resolution drift too high: {relative:.3%}"
print(f"WG-5 moisture resolution acceptance: L6={l6[0]:.3f} L7={l7[0]:.3f} drift={relative:.3%}")
PY
''')

# Calibration helper documents a dedicated resolution mode without making normal standard sweeps slower.
p = Path("scripts/run-wg5-calibration.sh")
s = p.read_text()
s = s.replace(
    "  quality)\n    bash \"$0\" standard\n    run_case \"ci-wg5-l7\" 5 7 24\n    ;;",
    "  resolution)\n    bash scripts/check-wg5-moisture-resolution.sh\n    ;;\n  quality)\n    bash \"$0\" standard\n    bash scripts/check-wg5-moisture-resolution.sh\n    ;;",
    1,
)
s = s.replace("usage: $0 [smoke|standard|quality]", "usage: $0 [smoke|standard|resolution|quality]")
p.write_text(s)
