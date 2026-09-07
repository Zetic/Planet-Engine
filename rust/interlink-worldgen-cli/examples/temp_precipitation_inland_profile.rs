use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history, generate_initial_topography,
    generate_lithosphere, generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    ClimateRequest, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest,
    TopographyRequest,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const SEEDS: &[&str] = &[
    "3", "interlink-wg7c", "wg4-boundary-a", "wg4-boundary-b", "wg4-boundary-c", "wg4-boundary-d",
];

#[derive(Clone, Copy)]
struct Entry { distance_m: f64, sample: u32 }
impl PartialEq for Entry { fn eq(&self, other: &Self) -> bool { self.distance_m.to_bits() == other.distance_m.to_bits() && self.sample == other.sample } }
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance_m.total_cmp(&self.distance_m).then_with(|| other.sample.cmp(&self.sample))
    }
}

fn coast_distances(topology: &interlink_worldgen::GeodesicTopology, submerged: &[u8], radius_m: f64) -> Vec<f64> {
    let n = topology.metrics().sample_count as usize;
    let mut distance = vec![f64::INFINITY; n];
    let mut heap = BinaryHeap::new();
    for i in 0..n {
        if submerged[i] != 0 {
            distance[i] = 0.0;
            heap.push(Entry { distance_m: 0.0, sample: i as u32 });
        }
    }
    while let Some(entry) = heap.pop() {
        let i = entry.sample as usize;
        if entry.distance_m > distance[i] + 1e-6 { continue; }
        for (neighbor, arc) in topology.neighbors_of(entry.sample).iter().zip(topology.neighbor_arc_lengths_of(entry.sample)) {
            let ni = *neighbor as usize;
            let candidate = entry.distance_m + *arc * radius_m;
            if candidate + 1e-6 < distance[ni] {
                distance[ni] = candidate;
                heap.push(Entry { distance_m: candidate, sample: *neighbor });
            }
        }
    }
    distance
}

fn budyko_aet(p: f64, pet: f64) -> f64 {
    if p <= 0.0 || pet <= 0.0 { return 0.0; }
    let omega = 2.6_f64;
    let phi = pet / p;
    let f = 1.0 + phi - (1.0 + phi.powf(omega)).powf(1.0 / omega);
    (p * f).clamp(0.0, p.min(pet))
}

fn main() -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let bands = [
        ("0-250", 0.0, 250_000.0),
        ("250-500", 250_000.0, 500_000.0),
        ("500-1000", 500_000.0, 1_000_000.0),
        ("1000-2000", 1_000_000.0, 2_000_000.0),
        (">2000", 2_000_000.0, f64::INFINITY),
    ];
    for seed in SEEDS {
        let coarse = build_icosphere(5).map_err(|e| e.to_string())?;
        let fine = build_icosphere(7).map_err(|e| e.to_string())?;
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(*seed, 16), planet).map_err(|e| e.to_string())?;
        let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(*seed), planet).map_err(|e| e.to_string())?;
        let lithosphere = generate_lithosphere(&coarse, &tectonics, &geology, &LithosphereRequest::new(*seed)).map_err(|e| e.to_string())?;
        let inherited = inherit_physical_state(&fine, 5, &tectonics, &geology, &lithosphere, planet).map_err(|e| e.to_string())?;
        let boundaries = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids).map_err(|e| e.to_string())?;
        let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &TopographyRequest::new(*seed)).map_err(|e| e.to_string())?;
        let climate = generate_coupled_climate(&fine, &terrain, planet, &ClimateRequest::new(*seed)).map_err(|e| e.to_string())?;
        let distance = coast_distances(&fine, &terrain.submerged_mask, planet.radius_m);
        let mut total_land_area = 0.0;
        let mut total_p_mass = 0.0;
        for i in 0..fine.metrics().sample_count as usize {
            if terrain.submerged_mask[i] != 0 { continue; }
            let a = fine.dual_area_steradians()[i];
            total_land_area += a;
            total_p_mass += f64::from(climate.annual_precipitation_mm[i]) * a;
        }
        println!("SEED {seed}");
        for (name, lo, hi) in bands {
            let mut area = 0.0;
            let mut p_sum = 0.0;
            let mut pet_sum = 0.0;
            let mut aet_sum = 0.0;
            let mut runoff_sum = 0.0;
            for i in 0..fine.metrics().sample_count as usize {
                if terrain.submerged_mask[i] != 0 || distance[i] < lo || distance[i] >= hi { continue; }
                let a = fine.dual_area_steradians()[i];
                let p = f64::from(climate.annual_precipitation_mm[i]);
                let pet = f64::from(climate.potential_evaporation_mm[i]);
                let aet = budyko_aet(p, pet);
                area += a; p_sum += p*a; pet_sum += pet*a; aet_sum += aet*a; runoff_sum += (p-aet).max(0.0)*a;
            }
            if area > 0.0 {
                println!("  {name:<10} area={:.1}% precip_share={:.1}% P={:.0} PET={:.0} AET={:.0} R={:.0} runoff={:.1}%",
                    area/total_land_area*100.0, p_sum/total_p_mass.max(1e-18)*100.0,
                    p_sum/area, pet_sum/area, aet_sum/area, runoff_sum/area,
                    runoff_sum/p_sum.max(1e-18)*100.0);
            }
        }
        let mut ranked = (0..fine.metrics().sample_count as usize)
            .filter(|&i| terrain.submerged_mask[i] == 0)
            .map(|i| (f64::from(climate.annual_precipitation_mm[i]), fine.dual_area_steradians()[i], distance[i]))
            .collect::<Vec<_>>();
        ranked.sort_by(|a,b| b.0.total_cmp(&a.0));
        let target_area = total_land_area * 0.10;
        let mut selected_area = 0.0;
        let mut d_sum = 0.0;
        let mut p_sum = 0.0;
        for (p,a,d) in ranked {
            if selected_area >= target_area { break; }
            selected_area += a; d_sum += d*a; p_sum += p*a;
        }
        println!("  wettest10 mean_coast_km={:.0} mean_P={:.0}", d_sum/selected_area.max(1e-18)/1000.0, p_sum/selected_area.max(1e-18));
    }
    Ok(())
}
