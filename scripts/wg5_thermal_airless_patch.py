from pathlib import Path

p = Path("rust/interlink-worldgen/src/climate.rs")
s = p.read_text()

old = '''                let effective_albedo = effective_shortwave_albedo(
                    physical.atmospheric_shortwave_reflectivity,
                    parameters.surface_albedo_shortwave_coupling,
                    albedo,
                );
'''
new = '''                let effective_albedo = if atmosphere_exists {
                    effective_shortwave_albedo(
                        physical.atmospheric_shortwave_reflectivity,
                        parameters.surface_albedo_shortwave_coupling,
                        albedo,
                    )
                } else {
                    // With no atmosphere there is no unresolved atmospheric/cloud
                    // shortwave masking: the exposed surface albedo is the TOA albedo.
                    albedo
                };
'''
if old not in s:
    raise SystemExit("missing effective shortwave caller anchor")
s = s.replace(old, new, 1)

old = '''            sea_surface_temperature = next_sst;
            for i in 0..sample_count {
                if ocean[i] {
                    exchange_air_sea_heat(
                        &mut temperature[i],
                        &mut sea_surface_temperature[i],
                        pressure[i],
                        planet,
                        physical,
                        parameters,
                        phase_seconds,
                    );
                }
            }
'''
new = '''            sea_surface_temperature = next_sst;
            for i in 0..sample_count {
                if !ocean[i] {
                    continue;
                }
                if atmosphere_exists {
                    exchange_air_sea_heat(
                        &mut temperature[i],
                        &mut sea_surface_temperature[i],
                        pressure[i],
                        planet,
                        physical,
                        parameters,
                        phase_seconds,
                    );
                } else {
                    // The temperature field is the exposed radiative surface state
                    // on an airless body, so there is no distinct air/SST reservoir.
                    sea_surface_temperature[i] = temperature[i];
                }
            }
'''
if old not in s:
    raise SystemExit("missing air-sea exchange loop anchor")
s = s.replace(old, new, 1)
p.write_text(s)

p = Path("rust/interlink-worldgen/tests/climate_ensemble.rs")
s = p.read_text()
old = '''    assert!(climate
        .annual_precipitation_mm
        .iter()
        .all(|value| *value == 0.0));

    let mut no_transport_request = ClimateRequest::new("wg5-airless");
'''
new = '''    assert!(climate
        .annual_precipitation_mm
        .iter()
        .all(|value| *value == 0.0));
    for (index, submerged) in terrain.submerged_mask.iter().enumerate() {
        if *submerged != 0 {
            assert!(
                (climate.sea_surface_temperature_mean_k[index]
                    - climate.temperature_mean_k[index])
                    .abs()
                    < 1.0e-4,
                "airless ocean surfaces must not retain a distinct atmospheric temperature reservoir",
            );
        }
    }

    let mut no_transport_request = ClimateRequest::new("wg5-airless");
'''
if old not in s:
    raise SystemExit("missing airless ensemble assertion anchor")
s = s.replace(old, new, 1)
p.write_text(s)

p = Path("docs/worldgen-rewrite/WG5_CLIMATE.md")
s = p.read_text()
anchor = "This is a reduced cloud/atmosphere masking term, not an explicit cloud field."
replacement = "This is a reduced cloud/atmosphere masking term, not an explicit cloud field. When reference atmospheric pressure is zero, that masking vanishes and the exposed surface albedo directly controls absorbed shortwave; ocean SST likewise collapses to the exposed radiative surface temperature because no distinct atmospheric air-sea reservoir exists."
if anchor not in s:
    raise SystemExit("missing WG5 airless documentation anchor")
s = s.replace(anchor, replacement, 1)
p.write_text(s)
