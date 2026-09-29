use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_initial_topography,
    generate_lithosphere_from_history, inherit_boundary_interfaces, inherit_physical_state,
    CrustKind, HistoricalLithosphereRequest, InheritedStructureKind, LithosphereRequest,
    PlanetPhysicalParameters, TopographyParameters, TopographyRequest,
};

#[derive(Default)]
struct WeightedStats {
    area: f64,
    elevation_sum: f64,
    above_1km: f64,
    above_2km: f64,
    above_3km: f64,
    above_4km: f64,
    samples: Vec<(f64, f64)>,
}

impl WeightedStats {
    fn add(&mut self, area: f64, elevation_m: f64) {
        self.area += area;
        self.elevation_sum += area * elevation_m;
        self.above_1km += area * f64::from(elevation_m >= 1_000.0);
        self.above_2km += area * f64::from(elevation_m >= 2_000.0);
        self.above_3km += area * f64::from(elevation_m >= 3_000.0);
        self.above_4km += area * f64::from(elevation_m >= 4_000.0);
        self.samples.push((elevation_m, area));
    }

    fn mean(&self) -> f64 {
        self.elevation_sum / self.area.max(1.0e-12)
    }

    fn fraction_above(&self, threshold_m: f64) -> f64 {
        let area = match threshold_m as i32 {
            1_000 => self.above_1km,
            2_000 => self.above_2km,
            3_000 => self.above_3km,
            4_000 => self.above_4km,
            _ => 0.0,
        };
        area / self.area.max(1.0e-12)
    }

    fn quantile(&mut self, q: f64) -> f64 {
        self.samples.sort_by(|left, right| left.0.total_cmp(&right.0));
        let target = self.area.max(1.0e-12) * q;
        let mut cumulative = 0.0;
        for (value, area) in &self.samples {
            cumulative += *area;
            if cumulative >= target {
                return *value;
            }
        }
        self.samples.last().map(|sample| sample.0).unwrap_or(0.0)
    }
}

fn inherited_orogen_response(
    inherited: &interlink_worldgen::InheritedPhysicalState,
    index: usize,
) -> f64 {
    let history = f64::from(inherited.orogenic_history[index]).clamp(0.0, 1.0);
    if history <= 0.0 {
        return 0.0;
    }
    let te =
        ((f64::from(inherited.effective_elastic_thickness_km[index]) - 4.0) / 82.0)
            .clamp(0.0, 1.0);
    let weakness = f64::from(inherited.weakness_index[index]).clamp(0.0, 1.0);
    let thickness =
        ((f64::from(inherited.crust_thickness_km[index]) - 28.0) / 28.0)
            .clamp(0.0, 1.0);
    let fabric = f64::from(inherited.structural_fabric_strength[index]).clamp(0.0, 1.0);
    let suture = f64::from(
        inherited.structural_zone_kind[index] == InheritedStructureKind::PaleoSuture as u8,
    );
    let exponent =
        (1.30 + 0.22 * weakness - 0.16 * te - 0.08 * thickness + 0.08 * suture * fabric)
            .clamp(1.10, 1.55);
    let support =
        (0.94 + 0.08 * te + 0.07 * thickness + 0.10 * suture * fabric
            + 0.02 * (1.0 - suture) * fabric)
            .clamp(0.90, 1.15);
    let shaped = history.powf(exponent) * support;
    history * 0.40 + shaped * 0.60
}

#[derive(Default)]
struct ComponentBudget {
    area: f64,
    base_isostatic: f64,
    historical_support: f64,
    thermal: f64,
    orogen: f64,
    ridge: f64,
    rift_basin: f64,
    trench: f64,
    arc: f64,
    mantle: f64,
}

impl ComponentBudget {
    fn add(
        &mut self,
        area: f64,
        base_isostatic: f64,
        historical_support: f64,
        thermal: f64,
        orogen: f64,
        ridge: f64,
        rift_basin: f64,
        trench: f64,
        arc: f64,
        mantle: f64,
    ) {
        self.area += area;
        self.base_isostatic += area * base_isostatic;
        self.historical_support += area * historical_support;
        self.thermal += area * thermal;
        self.orogen += area * orogen;
        self.ridge += area * ridge;
        self.rift_basin += area * rift_basin;
        self.trench += area * trench;
        self.arc += area * arc;
        self.mantle += area * mantle;
    }

    fn print(&self, seed: &str, label: &str) {
        let area = self.area.max(1.0e-12);
        println!(
            "hypsometry-budget seed={seed} class={label} base-iso={:.0} support={:.0} thermal={:.0} orogen={:.0} ridge={:.0} rift-basin={:.0} trench={:.0} arc={:.0} mantle={:.0}m",
            self.base_isostatic / area,
            self.historical_support / area,
            self.thermal / area,
            self.orogen / area,
            self.ridge / area,
            self.rift_basin / area,
            self.trench / area,
            self.arc / area,
            self.mantle / area,
        );
    }
}

fn print_variant(
    seed: &str,
    label: &str,
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    parameters: TopographyParameters,
) -> Result<(), String> {
    let terrain = generate_initial_topography(
        topology,
        inherited,
        boundaries,
        planet,
        &TopographyRequest {
            seed: seed.to_owned(),
            parameters,
        },
    )
    .map_err(|error| error.to_string())?;
    let mut land_area = 0.0_f64;
    let mut above_2km = 0.0_f64;
    let mut above_3km = 0.0_f64;
    for sample in 0..terrain.solid_elevation_m.len() {
        if terrain.submerged_mask[sample] != 0 {
            continue;
        }
        let area = topology.dual_area_steradians()[sample];
        let elevation = f64::from(terrain.elevation_above_sea_level_m[sample]);
        land_area += area;
        above_2km += area * f64::from(elevation >= 2_000.0);
        above_3km += area * f64::from(elevation >= 3_000.0);
    }
    println!(
        "hypsometry-variant seed={seed} variant={label} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p95={:.0}m max={:.0}m land>2/3km={:.1}/{:.1}% collision={:.0}m/{:.0}km inherited={:.0}m iso={:.2}",
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.mean_water_depth_m,
        terrain.metrics.p95_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
        above_2km / land_area.max(1.0e-12) * 100.0,
        above_3km / land_area.max(1.0e-12) * 100.0,
        parameters.collision_uplift_scale_m,
        parameters.collision_width_m / 1_000.0,
        parameters.inherited_orogeny_scale_m,
        parameters.isostatic_scale,
    );
    Ok(())
}

fn verify_seed(seed: &str) -> Result<(), String> {
    let coarse_level = 4;
    let fine_level = 6;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let parameters = TopographyParameters::default();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let frontend = generate_historical_frontend(
        &coarse,
        &HistoricalLithosphereRequest::new(seed, 16),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere_from_history(
        &coarse,
        &frontend.historical,
        &frontend.tectonics,
        &frontend.geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        coarse_level,
        &frontend.tectonics,
        &frontend.geology,
        &lithosphere,
        planet,
    )
    .map_err(|error| error.to_string())?;
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &frontend.tectonics,
        &frontend.geology,
        &inherited.plate_ids,
    )
    .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;

    let areas = fine.dual_area_steradians();
    let total_area = areas.iter().sum::<f64>().max(1.0e-12);
    let mut land = WeightedStats::default();
    let mut stable_land = WeightedStats::default();
    let mut modified_land = WeightedStats::default();
    let mut continental_land = WeightedStats::default();
    let mut stable_budget = ComponentBudget::default();
    let mut modified_budget = ComponentBudget::default();
    let mut continental_budget = ComponentBudget::default();
    let mut oceanic_budget = ComponentBudget::default();
    let mut continental_history_area = 0.0_f64;
    let mut continental_orogen_history_sum = 0.0_f64;
    let mut continental_thickness_sum = 0.0_f64;
    let mut continental_density_sum = 0.0_f64;
    let mut orogen_history_gt_025 = 0.0_f64;
    let mut orogen_history_gt_050 = 0.0_f64;
    let mut orogen_history_gt_075 = 0.0_f64;
    let mut inherited_orogen_sum = 0.0_f64;
    let mut active_collision_sum = 0.0_f64;
    let mut active_collision_gt_500 = 0.0_f64;
    let mut active_collision_gt_1000 = 0.0_f64;
    let mut active_collision_gt_2000 = 0.0_f64;
    let mut province_area = [0.0_f64; 7];
    let mut province_relief_sum = [0.0_f64; 7];

    for sample in 0..terrain.solid_elevation_m.len() {
        let area = areas[sample];
        let mantle_density = planet.isostatic_mantle_density_kg_per_m3;
        let thickness_m = f64::from(inherited.crust_thickness_km[sample]) * 1_000.0;
        let crust_density = f64::from(inherited.crust_density_kg_per_m3[sample]);
        let base_isostatic = thickness_m * (mantle_density - crust_density) / mantle_density
            * parameters.isostatic_scale;
        let historical_support =
            f64::from(terrain.isostatic_elevation_m[sample]) - base_isostatic;

        let province_kind = usize::from(inherited.province_kind[sample]).min(6);
        province_area[province_kind] += area;
        province_relief_sum[province_kind] +=
            area * f64::from(terrain.orogenic_elevation_m[sample]);
        let is_continental = inherited.crust_kind[sample] == CrustKind::Continental as u8;
        let stable = is_continental
            && inherited.structural_zone_kind[sample]
                != InheritedStructureKind::ContinentalMargin as u8
            && inherited.structural_zone_kind[sample] != InheritedStructureKind::InheritedRift as u8
            && inherited.rift_history[sample] < 0.22
            && inherited.subsidence_history[sample] < 0.28
            && inherited.basin_potential[sample] < 0.32;

        if is_continental {
            continental_history_area += area;
            let orogen_history = f64::from(inherited.orogenic_history[sample]);
            continental_orogen_history_sum += area * orogen_history;
            continental_thickness_sum += area * f64::from(inherited.crust_thickness_km[sample]);
            continental_density_sum += area * crust_density;
            orogen_history_gt_025 += area * f64::from(orogen_history >= 0.25);
            orogen_history_gt_050 += area * f64::from(orogen_history >= 0.50);
            orogen_history_gt_075 += area * f64::from(orogen_history >= 0.75);
            let inherited_orogen =
                parameters.inherited_orogeny_scale_m * inherited_orogen_response(&inherited, sample);
            let active_collision =
                (f64::from(terrain.orogenic_elevation_m[sample]) - inherited_orogen).max(0.0);
            inherited_orogen_sum += area * inherited_orogen;
            active_collision_sum += area * active_collision;
            active_collision_gt_500 += area * f64::from(active_collision >= 500.0);
            active_collision_gt_1000 += area * f64::from(active_collision >= 1_000.0);
            active_collision_gt_2000 += area * f64::from(active_collision >= 2_000.0);
            continental_budget.add(
                area,
                base_isostatic,
                historical_support,
                f64::from(terrain.thermal_elevation_m[sample]),
                f64::from(terrain.orogenic_elevation_m[sample]),
                f64::from(terrain.ridge_elevation_m[sample]),
                f64::from(terrain.rift_basin_elevation_m[sample]),
                f64::from(terrain.trench_elevation_m[sample]),
                f64::from(terrain.arc_elevation_m[sample]),
                f64::from(terrain.mantle_dynamic_elevation_m[sample]),
            );
            if stable {
                stable_budget.add(
                    area,
                    base_isostatic,
                    historical_support,
                    f64::from(terrain.thermal_elevation_m[sample]),
                    f64::from(terrain.orogenic_elevation_m[sample]),
                    f64::from(terrain.ridge_elevation_m[sample]),
                    f64::from(terrain.rift_basin_elevation_m[sample]),
                    f64::from(terrain.trench_elevation_m[sample]),
                    f64::from(terrain.arc_elevation_m[sample]),
                    f64::from(terrain.mantle_dynamic_elevation_m[sample]),
                );
            } else {
                modified_budget.add(
                    area,
                    base_isostatic,
                    historical_support,
                    f64::from(terrain.thermal_elevation_m[sample]),
                    f64::from(terrain.orogenic_elevation_m[sample]),
                    f64::from(terrain.ridge_elevation_m[sample]),
                    f64::from(terrain.rift_basin_elevation_m[sample]),
                    f64::from(terrain.trench_elevation_m[sample]),
                    f64::from(terrain.arc_elevation_m[sample]),
                    f64::from(terrain.mantle_dynamic_elevation_m[sample]),
                );
            }
        } else if inherited.crust_kind[sample] == CrustKind::Oceanic as u8 {
            oceanic_budget.add(
                area,
                base_isostatic,
                historical_support,
                f64::from(terrain.thermal_elevation_m[sample]),
                f64::from(terrain.orogenic_elevation_m[sample]),
                f64::from(terrain.ridge_elevation_m[sample]),
                f64::from(terrain.rift_basin_elevation_m[sample]),
                f64::from(terrain.trench_elevation_m[sample]),
                f64::from(terrain.arc_elevation_m[sample]),
                f64::from(terrain.mantle_dynamic_elevation_m[sample]),
            );
        }

        if terrain.submerged_mask[sample] == 0 {
            let elevation = f64::from(terrain.elevation_above_sea_level_m[sample]);
            land.add(area, elevation);
            if is_continental {
                continental_land.add(area, elevation);
                if stable {
                    stable_land.add(area, elevation);
                } else {
                    modified_land.add(area, elevation);
                }
            }
        }
    }

    println!(
        "hypsometry seed={seed} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p95={:.0}m max={:.0}m land>1/2/3/4km={:.1}/{:.1}/{:.1}/{:.1}%",
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.mean_water_depth_m,
        terrain.metrics.p95_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
        land.fraction_above(1_000.0) * 100.0,
        land.fraction_above(2_000.0) * 100.0,
        land.fraction_above(3_000.0) * 100.0,
        land.fraction_above(4_000.0) * 100.0,
    );
    println!(
        "hypsometry-land seed={seed} all p50/p90/p95={:.0}/{:.0}/{:.0}m continental mean={:.0} p50/p90/p95={:.0}/{:.0}/{:.0}m stable mean={:.0} modified mean={:.0}",
        land.quantile(0.50),
        land.quantile(0.90),
        land.quantile(0.95),
        continental_land.mean(),
        continental_land.quantile(0.50),
        continental_land.quantile(0.90),
        continental_land.quantile(0.95),
        stable_land.mean(),
        modified_land.mean(),
    );
    continental_budget.print(seed, "continental");
    stable_budget.print(seed, "stable");
    modified_budget.print(seed, "modified");
    oceanic_budget.print(seed, "oceanic");
    let continental_history_area = continental_history_area.max(1.0e-12);
    println!(
        "hypsometry-history seed={seed} continental-orogen-history-mean={:.3} area>=.25/.50/.75={:.1}/{:.1}/{:.1}% thickness={:.1}km density={:.0}kg/m3 inherited-orogen={:.0}m active-collision={:.0}m active>=.5/1/2km={:.1}/{:.1}/{:.1}%",
        continental_orogen_history_sum / continental_history_area,
        orogen_history_gt_025 / continental_history_area * 100.0,
        orogen_history_gt_050 / continental_history_area * 100.0,
        orogen_history_gt_075 / continental_history_area * 100.0,
        continental_thickness_sum / continental_history_area,
        continental_density_sum / continental_history_area,
        inherited_orogen_sum / continental_history_area,
        active_collision_sum / continental_history_area,
        active_collision_gt_500 / continental_history_area * 100.0,
        active_collision_gt_1000 / continental_history_area * 100.0,
        active_collision_gt_2000 / continental_history_area * 100.0,
    );
    for kind in 0..=6 {
        if province_area[kind] > 0.0 {
            println!(
                "hypsometry-province seed={seed} kind={kind} area={:.1}% mean-relief={:.0}m",
                province_area[kind] / total_area * 100.0,
                province_relief_sum[kind] / province_area[kind],
            );
        }
    }
    println!(
        "hypsometry-area seed={seed} continental={:.1}% stable={:.1}% modified={:.1}% water-closure={:.3e}",
        continental_budget.area / total_area * 100.0,
        stable_budget.area / total_area * 100.0,
        modified_budget.area / total_area * 100.0,
        terrain.metrics.water_volume_relative_error,
    );

    let mut candidate = TopographyParameters::default();
    candidate.isostatic_scale = 0.50;
    candidate.collision_uplift_scale_m = 2_200.0;
    print_variant(seed, "iso50-c2200", &fine, &inherited, &boundaries, planet, candidate)?;

    let mut candidate = TopographyParameters::default();
    candidate.isostatic_scale = 0.45;
    candidate.collision_uplift_scale_m = 2_200.0;
    print_variant(seed, "iso45-c2200", &fine, &inherited, &boundaries, planet, candidate)?;

    let mut candidate = TopographyParameters::default();
    candidate.isostatic_scale = 0.45;
    candidate.collision_uplift_scale_m = 2_000.0;
    print_variant(seed, "iso45-c2000", &fine, &inherited, &boundaries, planet, candidate)?;

    let mut candidate = TopographyParameters::default();
    candidate.isostatic_scale = 0.42;
    candidate.collision_uplift_scale_m = 2_000.0;
    print_variant(seed, "iso42-c2000", &fine, &inherited, &boundaries, planet, candidate)?;

    Ok(())
}

fn main() -> Result<(), String> {
    for seed in [
        "interlink-wg7c",
        "1",
        "2",
        "continental-hypsometry-holdout",
    ] {
        verify_seed(seed)?;
    }
    Ok(())
}
