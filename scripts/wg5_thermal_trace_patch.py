from pathlib import Path

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()
old = '''        let mut squared_change = 0.0;
        for i in 0..sample_count {
            let delta_temperature = temperature[i] - start_temperature[i];
            let delta_sst = sea_surface_temperature[i] - start_sst[i];
            squared_change += delta_temperature * delta_temperature + 0.5 * delta_sst * delta_sst;
        }
        final_temperature_rms_change = (squared_change / (sample_count as f64 * 1.5)).sqrt();
        spinup_years = year + 1;
'''
new = '''        let mut squared_change = 0.0;
        let mut squared_temperature_change = 0.0;
        let mut squared_sst_change = 0.0;
        let mut ocean_count = 0usize;
        for i in 0..sample_count {
            let delta_temperature = temperature[i] - start_temperature[i];
            let delta_sst = sea_surface_temperature[i] - start_sst[i];
            squared_temperature_change += delta_temperature * delta_temperature;
            if ocean[i] {
                squared_sst_change += delta_sst * delta_sst;
                ocean_count += 1;
            }
            squared_change += delta_temperature * delta_temperature + 0.5 * delta_sst * delta_sst;
        }
        final_temperature_rms_change = (squared_change / (sample_count as f64 * 1.5)).sqrt();
        let atmosphere_rms = (squared_temperature_change / sample_count as f64).sqrt();
        let sst_rms = if ocean_count > 0 {
            (squared_sst_change / ocean_count as f64).sqrt()
        } else {
            0.0
        };
        spinup_years = year + 1;
        eprintln!(
            "WG5_THERMAL_TRACE year={} combined_rms={:.9} atmosphere_rms={:.9} sst_rms={:.9}",
            spinup_years, final_temperature_rms_change, atmosphere_rms, sst_rms
        );
'''
if old not in s:
    raise SystemExit('missing convergence trace anchor')
p.write_text(s.replace(old, new, 1))
