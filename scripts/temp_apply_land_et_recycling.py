from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

old = '''    let mut requested_ocean_evaporation_mass = vec![0.0; sample_count];

    for year in 0..parameters.maximum_spinup_years {'''
new = '''    let mut requested_ocean_evaporation_mass = vec![0.0; sample_count];
    // Experimental annual land-ET recycling closure. Year one is ocean-forced;
    // later years use the previous annual P/PET Budyko partition to bound the
    // fraction of current land PET that can return to atmospheric moisture.
    let mut land_evaporation_recycling_fraction = vec![0.0_f64; sample_count];

    for year in 0..parameters.maximum_spinup_years {'''
assert old in text
text = text.replace(old, new, 1)

old = '''                    potential_evaporation_mass_year[i] += diagnostic_potential_mass;
                    if ocean[i] {'''
new = '''                    potential_evaporation_mass_year[i] += diagnostic_potential_mass;
                    if !ocean[i] {
                        let land_evaporation_mass = diagnostic_potential_mass
                            * land_evaporation_recycling_fraction[i];
                        moisture_mass[i] += land_evaporation_mass;
                        phase_evaporation += land_evaporation_mass;
                    }
                    if ocean[i] {'''
assert old in text
text = text.replace(old, new, 1)

old = '''        let mut squared_change = 0.0;
        for i in 0..sample_count {'''
new = '''        // Close the annual land surface partition for the next spin-up year.
        // Fu/Budyko omega intentionally matches the accepted WG-6B default.
        for i in 0..sample_count {
            if ocean[i] {
                land_evaporation_recycling_fraction[i] = 0.0;
                continue;
            }
            let area = cell_area_m2[i].max(1.0);
            let precipitation_mm = precipitation_mass_year[i] / area;
            let pet_mm = potential_evaporation_mass_year[i] / area;
            if precipitation_mm <= 0.0 || pet_mm <= 0.0 {
                land_evaporation_recycling_fraction[i] = 0.0;
                continue;
            }
            let omega = 2.6_f64;
            let aridity = pet_mm / precipitation_mm;
            let aet_fraction = 1.0 + aridity
                - (1.0 + aridity.powf(omega)).powf(1.0 / omega);
            let aet_mm = (precipitation_mm * aet_fraction)
                .clamp(0.0, precipitation_mm.min(pet_mm));
            land_evaporation_recycling_fraction[i] = (aet_mm / pet_mm).clamp(0.0, 1.0);
        }

        let mut squared_change = 0.0;
        for i in 0..sample_count {'''
assert old in text
text = text.replace(old, new, 1)

path.write_text(text)
