from pathlib import Path

p = Path("src/worldgen/worldgenWorker.ts")
s = p.read_text()
replacements = [
    (
        "moisture_budget_relative_error(): number;",
        "moisture_budget_relative_error(): number; moisture_transport_limiter_fraction(): number; maximum_moisture_transport_substeps(): number;",
        "moisture transport metrics",
    ),
    (
        "annual_precipitation_mm(): Float32Array;",
        "annual_precipitation_mm(): Float32Array; precipitation_phase_rate_mm_year(): Float32Array;",
        "seasonal precipitation diagnostic",
    ),
]
for old, new, label in replacements:
    if new in s:
        continue
    if old not in s:
        raise SystemExit(f"missing WasmClimate interface anchor for {label}")
    s = s.replace(old, new, 1)

for required in [
    "precipitation_phase_rate_mm_year(): Float32Array;",
    "moisture_transport_limiter_fraction(): number;",
    "maximum_moisture_transport_substeps(): number;",
]:
    if required not in s:
        raise SystemExit(f"WasmClimate interface still missing {required}")

p.write_text(s)
print("Fixed browser WasmClimate interface additions")
