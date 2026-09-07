use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, ClimateRequest, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
};

const SEEDS: &[&str] = &[
    "3",
    "interlink-wg7c",
    "wg4-boundary-a",
    "wg4-boundary-b",
    "wg4-boundary-c",
    "wg4-boundary-d",
];

fn weighted_corr(xs: &[f64], ys: &[f64], ws: &[f64]) -> f64 {
    let wsum = ws.iter().sum::<f64>().max(1.0e-18);
    let mx = xs.iter().zip(ws).map(|(x,w)| x*w).sum::<f64>() / wsum;
    let my = ys.iter().zip(ws).map(|(y,w)| y*w).sum::<f64>() / wsum;
    let mut cov = 0.0;
    let mut vx = 0.0;
    let mut vy = 0.0;
    for i in 0..xs.len() {
        let dx = xs[i] - mx;
        let dy = ys[i] - my;
        cov += ws[i] * dx * dy;
        vx += ws[i] * dx * dx;
        vy += ws[i] * dy * dy;
    }
    cov / (vx * vy).sqrt().max(1.0e-18)
}

fn main() -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;

    for seed in SEEDS {
        let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
        let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(*seed, plates), planet)
            .map_err(|e| e.to_string())?;
        let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(*seed), planet)
            .map_err(|e| e.to_string())?;
        let lithosphere = generate_lithosphere(&coarse, &tectonics, &geology, &LithosphereRequest::new(*seed))
            .map_err(|e| e.to_string())?;
        let inherited = inherit_physical_state(&fine, coarse_level, &tectonics, &geology, &lithosphere, planet)
            .map_err(|e| e.to_string())?;
        let boundaries = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .map_err(|e| e.to_string())?;
        let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &TopographyRequest::new(*seed))
            .map_err(|e| e.to_string())?;
        let climate = generate_coupled_climate(&fine, &terrain, planet, &ClimateRequest::new(*seed))
            .map_err(|e| e.to_string())?;

        let mut p = Vec::new();
        let mut pet = Vec::new();
        let mut t = Vec::new();
        let mut z = Vec::new();
        let mut w = Vec::new();
        let mut total_precip = 0.0_f64;
        let mut p_pet_lt025 = 0.0_f64;
        let mut p_pet_lt05 = 0.0_f64;
        let mut p_pet_lt1 = 0.0_f64;
        let mut p_cold273 = 0.0_f64;
        let mut p_cold280 = 0.0_f64;
        let mut p_high2k = 0.0_f64;
        let mut wet_area = 0.0_f64;
        let mut wet_p = 0.0_f64;
        let mut wet_pet = 0.0_f64;
        let mut wet_t = 0.0_f64;
        let mut wet_z = 0.0_f64;
        let mut very_wet_area = 0.0_f64;
        let mut very_wet_p = 0.0_f64;
        let mut very_wet_pet = 0.0_f64;
        let mut very_wet_t = 0.0_f64;

        for i in 0..fine.metrics().sample_count as usize {
            if terrain.submerged_mask[i] != 0 { continue; }
            let area = fine.dual_area_steradians()[i];
            let pi = f64::from(climate.annual_precipitation_mm[i]);
            let ei = f64::from(climate.potential_evaporation_mm[i]);
            let ti = f64::from(climate.temperature_mean_k[i]);
            let zi = f64::from(terrain.elevation_above_sea_level_m[i]).max(0.0);
            p.push(pi); pet.push(ei); t.push(ti); z.push(zi); w.push(area);
            let pmass = pi * area;
            total_precip += pmass;
            let ratio = if pi > 1.0 { ei / pi } else { f64::INFINITY };
            if ratio < 0.25 { p_pet_lt025 += pmass; }
            if ratio < 0.50 { p_pet_lt05 += pmass; }
            if ratio < 1.00 { p_pet_lt1 += pmass; }
            if ti < 273.15 { p_cold273 += pmass; }
            if ti < 280.0 { p_cold280 += pmass; }
            if zi > 2_000.0 { p_high2k += pmass; }
            if pi >= 1_000.0 {
                wet_area += area; wet_p += pi*area; wet_pet += ei*area; wet_t += ti*area; wet_z += zi*area;
            }
            if pi >= 2_000.0 {
                very_wet_area += area; very_wet_p += pi*area; very_wet_pet += ei*area; very_wet_t += ti*area;
            }
        }
        let land_area = w.iter().sum::<f64>();
        println!("SEED {seed}");
        println!("  corr P-PET={:.3} P-T={:.3} P-Z={:.3} PET-T={:.3}",
            weighted_corr(&p,&pet,&w), weighted_corr(&p,&t,&w), weighted_corr(&p,&z,&w), weighted_corr(&pet,&t,&w));
        println!("  precip_share PET/P<0.25={:.1}% <0.5={:.1}% <1={:.1}% T<273={:.1}% T<280={:.1}% Z>2km={:.1}%",
            p_pet_lt025/total_precip*100.0, p_pet_lt05/total_precip*100.0, p_pet_lt1/total_precip*100.0,
            p_cold273/total_precip*100.0, p_cold280/total_precip*100.0, p_high2k/total_precip*100.0);
        if wet_area > 0.0 {
            println!("  wet>=1000 area={:.1}% precip_share={:.1}% P={:.0} PET={:.0} PET/P={:.2} T={:.1}K Z={:.0}m",
                wet_area/land_area*100.0, wet_p/total_precip*100.0, wet_p/wet_area, wet_pet/wet_area,
                wet_pet/wet_p.max(1e-12), wet_t/wet_area, wet_z/wet_area);
        }
        if very_wet_area > 0.0 {
            println!("  wet>=2000 area={:.1}% precip_share={:.1}% P={:.0} PET={:.0} PET/P={:.2} T={:.1}K",
                very_wet_area/land_area*100.0, very_wet_p/total_precip*100.0, very_wet_p/very_wet_area,
                very_wet_pet/very_wet_area, very_wet_pet/very_wet_p.max(1e-12), very_wet_t/very_wet_area);
        }
    }
    Ok(())
}
