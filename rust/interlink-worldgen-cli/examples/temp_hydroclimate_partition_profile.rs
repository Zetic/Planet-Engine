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

fn budyko_aet(p: f64, pet: f64, omega: f64) -> f64 {
    if p <= 0.0 || pet <= 0.0 {
        return 0.0;
    }
    let phi = pet / p;
    let fraction = 1.0 + phi - (1.0 + phi.powf(omega)).powf(1.0 / omega);
    (p * fraction).clamp(0.0, p.min(pet))
}

fn main() -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let scenarios = [
        ("base", 1.00_f64, 2.6_f64),
        ("omega35", 1.00, 3.5),
        ("omega50", 1.00, 5.0),
        ("pet125", 1.25, 2.6),
        ("pet150", 1.50, 2.6),
        ("pet125_o35", 1.25, 3.5),
        ("pet150_o35", 1.50, 3.5),
        ("pet150_o50", 1.50, 5.0),
    ];

    let mut agg_area = vec![0.0_f64; scenarios.len()];
    let mut agg_p = vec![0.0_f64; scenarios.len()];
    let mut agg_pet = vec![0.0_f64; scenarios.len()];
    let mut agg_aet = vec![0.0_f64; scenarios.len()];
    let mut agg_runoff = vec![0.0_f64; scenarios.len()];

    for seed in SEEDS {
        let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
        let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
        let tectonics = generate_tectonics(
            &coarse,
            &TectonicsRequest::new(*seed, plates),
            planet,
        )
        .map_err(|e| e.to_string())?;
        let geology = generate_crust_and_history(
            &coarse,
            &tectonics,
            &GeologyRequest::new(*seed),
            planet,
        )
        .map_err(|e| e.to_string())?;
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(*seed),
        )
        .map_err(|e| e.to_string())?;
        let inherited = inherit_physical_state(
            &fine,
            coarse_level,
            &tectonics,
            &geology,
            &lithosphere,
            planet,
        )
        .map_err(|e| e.to_string())?;
        let boundaries = inherit_boundary_interfaces(
            &coarse,
            &fine,
            &tectonics,
            &geology,
            &inherited.plate_ids,
        )
        .map_err(|e| e.to_string())?;
        let terrain = generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(*seed),
        )
        .map_err(|e| e.to_string())?;
        let climate = generate_coupled_climate(
            &fine,
            &terrain,
            planet,
            &ClimateRequest::new(*seed),
        )
        .map_err(|e| e.to_string())?;

        let mut land_area = 0.0_f64;
        let mut temp_sum = 0.0_f64;
        let mut elev_sum = 0.0_f64;
        let mut cold_area = 0.0_f64;
        let mut high_area = 0.0_f64;
        let mut base_p_sum = 0.0_f64;
        let mut base_pet_sum = 0.0_f64;
        for i in 0..fine.metrics().sample_count as usize {
            if terrain.submerged_mask[i] != 0 {
                continue;
            }
            let area = fine.dual_area_steradians()[i];
            land_area += area;
            let t = f64::from(climate.temperature_mean_k[i]);
            let z = f64::from(terrain.elevation_above_sea_level_m[i]).max(0.0);
            temp_sum += t * area;
            elev_sum += z * area;
            base_p_sum += f64::from(climate.annual_precipitation_mm[i]) * area;
            base_pet_sum += f64::from(climate.potential_evaporation_mm[i]) * area;
            if t < 273.15 {
                cold_area += area;
            }
            if z > 2_000.0 {
                high_area += area;
            }
        }
        println!(
            "SEED {seed} land={:.2}% temp={:.2}K elev={:.0}m cold={:.2}% high2k={:.2}% precip={:.1} pet={:.1} pet_p={:.3}",
            terrain.metrics.land_area_fraction * 100.0,
            temp_sum / land_area,
            elev_sum / land_area,
            cold_area / land_area * 100.0,
            high_area / land_area * 100.0,
            base_p_sum / land_area,
            base_pet_sum / land_area,
            base_pet_sum / base_p_sum.max(1.0e-12),
        );

        for (s, (name, pet_scale, omega)) in scenarios.iter().enumerate() {
            let mut p_sum = 0.0_f64;
            let mut pet_sum = 0.0_f64;
            let mut aet_sum = 0.0_f64;
            let mut runoff_sum = 0.0_f64;
            for i in 0..fine.metrics().sample_count as usize {
                if terrain.submerged_mask[i] != 0 {
                    continue;
                }
                let area = fine.dual_area_steradians()[i];
                let p = f64::from(climate.annual_precipitation_mm[i]);
                let pet = f64::from(climate.potential_evaporation_mm[i]) * pet_scale;
                let aet = budyko_aet(p, pet, *omega);
                p_sum += p * area;
                pet_sum += pet * area;
                aet_sum += aet * area;
                runoff_sum += (p - aet).max(0.0) * area;
            }
            println!(
                "  {name:<12} pet_scale={pet_scale:.2} omega={omega:.1} P={:.1} PET={:.1} AET={:.1} R={:.1} runoff={:.1}%",
                p_sum / land_area,
                pet_sum / land_area,
                aet_sum / land_area,
                runoff_sum / land_area,
                runoff_sum / p_sum.max(1.0e-12) * 100.0,
            );
            agg_area[s] += land_area;
            agg_p[s] += p_sum;
            agg_pet[s] += pet_sum;
            agg_aet[s] += aet_sum;
            agg_runoff[s] += runoff_sum;
        }
    }

    println!("AGGREGATE");
    for (s, (name, pet_scale, omega)) in scenarios.iter().enumerate() {
        println!(
            "  {name:<12} pet_scale={pet_scale:.2} omega={omega:.1} P={:.1} PET={:.1} AET={:.1} R={:.1} runoff={:.1}%",
            agg_p[s] / agg_area[s],
            agg_pet[s] / agg_area[s],
            agg_aet[s] / agg_area[s],
            agg_runoff[s] / agg_area[s],
            agg_runoff[s] / agg_p[s].max(1.0e-12) * 100.0,
        );
    }
    Ok(())
}
