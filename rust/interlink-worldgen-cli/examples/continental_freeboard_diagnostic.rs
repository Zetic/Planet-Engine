use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_initial_topography,
    generate_lithosphere_from_history, inherit_boundary_interfaces, inherit_physical_state,
    CrustKind, HistoricalLithosphereRequest, InheritedStructureKind, LithosphereRequest,
    PlanetPhysicalParameters, TopographyRequest,
};

#[derive(Default)]
struct Bucket {
    area: f64,
    land_area: f64,
    elevation_area_sum: f64,
    isostatic_area_sum: f64,
    thickness_area_sum: f64,
    stability_area_sum: f64,
    orogen_area_sum: f64,
    below_500m: f64,
    below_1000m: f64,
    above_2000m: f64,
    samples: Vec<(f64, f64)>,
}

impl Bucket {
    fn add(
        &mut self,
        area: f64,
        land: bool,
        elevation_m: f64,
        isostatic_m: f64,
        thickness_km: f64,
        stability: f64,
        orogen_m: f64,
    ) {
        self.area += area;
        self.isostatic_area_sum += area * isostatic_m;
        self.thickness_area_sum += area * thickness_km;
        self.stability_area_sum += area * stability;
        self.orogen_area_sum += area * orogen_m;
        if land {
            self.land_area += area;
            self.elevation_area_sum += area * elevation_m;
            self.below_500m += area * f64::from(elevation_m < 500.0);
            self.below_1000m += area * f64::from(elevation_m < 1_000.0);
            self.above_2000m += area * f64::from(elevation_m >= 2_000.0);
            self.samples.push((elevation_m, area));
        }
    }

    fn q(&mut self, q: f64) -> f64 {
        self.samples.sort_by(|a, b| a.0.total_cmp(&b.0));
        let target = self.land_area.max(1.0e-12) * q;
        let mut cumulative = 0.0;
        for (value, area) in &self.samples {
            cumulative += *area;
            if cumulative >= target {
                return *value;
            }
        }
        self.samples.last().map(|v| v.0).unwrap_or(0.0)
    }

    fn print(&mut self, seed: &str, name: &str, total_area: f64) {
        let area = self.area.max(1.0e-12);
        let land = self.land_area.max(1.0e-12);
        let q25 = self.q(0.25);
        let q50 = self.q(0.50);
        let q75 = self.q(0.75);
        println!(
            "freeboard seed={seed} class={name} area={:.1}% emergent={:.1}% mean-land={:.0}m q25/50/75={:.0}/{:.0}/{:.0}m low<.5/1km={:.1}/{:.1}% high>=2km={:.1}% iso={:.0}m thickness={:.1}km stability={:.2} orogen={:.0}m",
            self.area / total_area * 100.0,
            self.land_area / area * 100.0,
            self.elevation_area_sum / land,
            q25,
            q50,
            q75,
            self.below_500m / land * 100.0,
            self.below_1000m / land * 100.0,
            self.above_2000m / land * 100.0,
            self.isostatic_area_sum / area,
            self.thickness_area_sum / area,
            self.stability_area_sum / area,
            self.orogen_area_sum / area,
        );
    }
}

fn run(seed: &str, coarse_level: u8, fine_level: u8) -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
    let frontend = generate_historical_frontend(
        &coarse,
        &HistoricalLithosphereRequest::new(seed, 16),
        planet,
    )
    .map_err(|e| e.to_string())?;
    let lithosphere = generate_lithosphere_from_history(
        &coarse,
        &frontend.historical,
        &frontend.tectonics,
        &frontend.geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        coarse_level,
        &frontend.tectonics,
        &frontend.geology,
        &lithosphere,
        planet,
    )
    .map_err(|e| e.to_string())?;
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &frontend.tectonics,
        &frontend.geology,
        &inherited.plate_ids,
    )
    .map_err(|e| e.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;

    let areas = fine.dual_area_steradians();
    let total_area = areas.iter().sum::<f64>().max(1.0e-12);
    let mut all_continental = Bucket::default();
    let mut quiet = Bucket::default();
    let mut modified = Bucket::default();
    let mut orogenic = Bucket::default();

    for sample in 0..terrain.solid_elevation_m.len() {
        if inherited.crust_kind[sample] != CrustKind::Continental as u8 {
            continue;
        }
        let area = areas[sample];
        let land = terrain.submerged_mask[sample] == 0;
        let elevation = if land {
            f64::from(terrain.elevation_above_sea_level_m[sample])
        } else {
            0.0
        };
        let isostatic = f64::from(terrain.isostatic_elevation_m[sample]);
        let thickness = f64::from(inherited.crust_thickness_km[sample]);
        let stability = f64::from(inherited.continental_stability_index[sample]);
        let orogen = f64::from(terrain.orogenic_elevation_m[sample]);

        all_continental.add(area, land, elevation, isostatic, thickness, stability, orogen);

        let structural_modified =
            inherited.structural_zone_kind[sample] == InheritedStructureKind::ContinentalMargin as u8
                || inherited.structural_zone_kind[sample] == InheritedStructureKind::InheritedRift as u8;
        let modified_state = structural_modified
            || inherited.rift_history[sample] >= 0.22
            || inherited.subsidence_history[sample] >= 0.28
            || inherited.basin_potential[sample] >= 0.32;
        let orogenic_state = inherited.province_kind[sample] != 0
            || inherited.orogenic_history[sample] >= 0.45
            || orogen >= 800.0;

        if orogenic_state {
            orogenic.add(area, land, elevation, isostatic, thickness, stability, orogen);
        } else if modified_state {
            modified.add(area, land, elevation, isostatic, thickness, stability, orogen);
        } else {
            quiet.add(area, land, elevation, isostatic, thickness, stability, orogen);
        }
    }

    println!(
        "freeboard-world seed={seed} L{coarse_level}->L{fine_level} land={:.1}% mean-land={:.0}m sea={:.0}m p95={:.0}m",
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.sea_level_m.unwrap_or(0.0),
        terrain.metrics.p95_solid_elevation_m,
    );
    all_continental.print(seed, "continental", total_area);
    quiet.print(seed, "quiet", total_area);
    modified.print(seed, "modified", total_area);
    orogenic.print(seed, "orogenic", total_area);
    Ok(())
}

fn main() -> Result<(), String> {
    for seed in ["interlink-wg7c", "1", "2", "continental-freeboard-holdout"] {
        run(seed, 4, 6)?;
    }
    for seed in ["interlink-wg7c", "1", "2", "continental-freeboard-holdout"] {
        run(seed, 6, 8)?;
    }
    Ok(())
}
