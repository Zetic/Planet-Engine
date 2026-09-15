from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing anchor in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


def replace_regex(path: str, pattern: str, replacement: str) -> None:
    p = Path(path)
    text = p.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex replacement in {path}, got {count}: {pattern[:100]!r}")
    p.write_text(updated)


orogen = 'rust/interlink-worldgen/src/orogen_provinces.rs'
replace_once(orogen,
    'pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 1;\nconst OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v1";',
    'pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 2;\nconst OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v2";')
replace_once(orogen,
    '    pub orogenic_intensity: Vec<f32>,\n    pub crustal_root_index: Vec<f32>,',
    '    pub orogenic_intensity: Vec<f32>,\n    /// Boundary-normal distance to the winning connected convergent source. Meaningful only where province_ids > 0.\n    pub boundary_distance_km: Vec<f32>,\n    /// Narrow high-relief orogenic/arc spine. This is intentionally much narrower than the full deformation province.\n    pub mountain_core_index: Vec<f32>,\n    /// Pre-orogenic resistance to inland deformation; high values represent strong/cratonic substrate.\n    pub interior_resistance_index: Vec<f32>,\n    pub crustal_root_index: Vec<f32>,')
replace_once(orogen,
    '    core_strength: f64,\n}',
    '    core_strength: f64,\n    plateau_eligibility: f64,\n}')

replace_regex(orogen,
    r'#\[derive\(Clone, Copy, Debug\)\]\nstruct DistanceFrontier \{.*?\nfn clamp01',
    '''#[derive(Clone, Copy, Debug)]
struct DistanceFrontier {
    cost_km: f64,
    physical_distance_km: f64,
    source_index: usize,
    sample: u32,
}
impl PartialEq for DistanceFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.cost_km.to_bits() == other.cost_km.to_bits()
            && self.physical_distance_km.to_bits() == other.physical_distance_km.to_bits()
            && self.source_index == other.source_index
            && self.sample == other.sample
    }
}
impl Eq for DistanceFrontier {}
impl Ord for DistanceFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost_km
            .total_cmp(&self.cost_km)
            .then_with(|| other.physical_distance_km.total_cmp(&self.physical_distance_km))
            .then_with(|| other.source_index.cmp(&self.source_index))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}
impl PartialOrd for DistanceFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn clamp01''')

replace_regex(orogen,
    r'fn province_kind\(segment: &\[usize\], edge_traits: &\[Option<EdgeTraits>\]\) -> OrogenProvinceKind \{.*?\n\}\n\nfn base_width_km',
    '''fn province_kind(segment: &[usize], edge_traits: &[Option<EdgeTraits>]) -> OrogenProvinceKind {
    let count = segment.len().max(1) as f64;
    let first = edge_traits[segment[0]].unwrap();
    let mean_obliquity = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().obliquity_deg)
        .sum::<f64>()
        / count;
    let mean_fragment = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().fragment_contact)
        .sum::<f64>()
        / count;
    let mean_age = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().age_myr)
        .sum::<f64>()
        / count;
    let mean_convergence = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().convergence_km)
        .sum::<f64>()
        / count;
    let mean_weakness = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().weakness)
        .sum::<f64>()
        / count;
    let mean_te_km = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().mean_te_km)
        .sum::<f64>()
        / count;
    let maturity = 0.42 * clamp01(mean_age / 95.0) + 0.58 * clamp01(mean_convergence / 3200.0);
    let shortening = clamp01(mean_convergence / 3000.0);

    if mean_obliquity >= 62.0 {
        return OrogenProvinceKind::TranspressionalOrogen;
    }
    match first.class {
        BoundaryClass::Cordilleran => OrogenProvinceKind::CordilleranArc,
        BoundaryClass::IslandArc => OrogenProvinceKind::IslandArc,
        BoundaryClass::Collision => {
            if mean_fragment >= 0.55 {
                OrogenProvinceKind::TerraneAccretion
            } else if maturity >= 0.66
                && shortening >= 0.60
                && mean_weakness >= 0.32
                && mean_te_km <= 72.0
            {
                // Tibetan-style broad plateaus are deliberately exceptional. A collision must be
                // mature, strongly shortened, and mechanically capable of transmitting strain.
                OrogenProvinceKind::CollisionalPlateau
            } else {
                OrogenProvinceKind::ContinentalCollision
            }
        }
    }
}

fn base_width_km''')

replace_regex(orogen,
    r'fn base_width_km\(kind: OrogenProvinceKind, maturity: f64, weakness: f64\) -> f64 \{.*?\n\}\n\nfn build_provinces_and_sources',
    '''fn base_width_km(kind: OrogenProvinceKind, maturity: f64, weakness: f64) -> f64 {
    // Width is now a deformation-reach scale, not a direct mountain footprint. Values are
    // intentionally narrow enough that strong continental interiors can halt strain within
    // hundreds of kilometres while exceptional mature plateaus can still exceed 1000 km total.
    match kind {
        OrogenProvinceKind::ContinentalCollision => 170.0 + maturity * 190.0 + weakness * 90.0,
        OrogenProvinceKind::CollisionalPlateau => 300.0 + maturity * 310.0 + weakness * 130.0,
        OrogenProvinceKind::CordilleranArc => 120.0 + maturity * 110.0 + weakness * 55.0,
        OrogenProvinceKind::IslandArc => 90.0 + maturity * 95.0 + weakness * 45.0,
        OrogenProvinceKind::TerraneAccretion => 135.0 + maturity * 145.0 + weakness * 75.0,
        OrogenProvinceKind::TranspressionalOrogen => 85.0 + maturity * 100.0 + weakness * 45.0,
    }
}

fn source_widths(
    traits: EdgeTraits,
    kind: OrogenProvinceKind,
    system_has_endpoints: bool,
) -> (f64, f64, f64, f64, f64, f64) {
    let maturity =
        0.42 * clamp01(traits.age_myr / 95.0) + 0.58 * clamp01(traits.convergence_km / 3200.0);
    let shortening = clamp01(traits.convergence_km / 3000.0);
    let local_control = (0.72
        + traits.weakness * 0.36
        + traits.fabric * 0.14
        + traits.age_discontinuity * 0.12
        + traits.province_boundary * 0.08
        + clamp01(traits.curvature_deg / 90.0) * 0.10
        + traits.fragment_contact * 0.08)
        .clamp(0.68, 1.42);
    let mut base = base_width_km(kind, maturity, traits.weakness) * local_control;

    let taper = if system_has_endpoints {
        smoothstep(traits.along_fraction / 0.16) * smoothstep((1.0 - traits.along_fraction) / 0.16)
    } else {
        1.0
    };
    base *= 0.24 + taper * 0.76;

    let (minimum, maximum) = match kind {
        OrogenProvinceKind::CollisionalPlateau => (220.0, 900.0),
        OrogenProvinceKind::ContinentalCollision => (120.0, 560.0),
        OrogenProvinceKind::TerraneAccretion => (100.0, 480.0),
        OrogenProvinceKind::CordilleranArc => (85.0, 380.0),
        OrogenProvinceKind::IslandArc => (65.0, 300.0),
        OrogenProvinceKind::TranspressionalOrogen => (55.0, 260.0),
    };
    base = base.clamp(minimum, maximum);

    let asymmetry = ((traits.strength_b - traits.strength_a) * 0.42
        + (traits.fabric - 0.5) * 0.08)
        .clamp(-0.28, 0.28);
    let mut width_a = (base * (1.0 + asymmetry)).clamp(minimum * 0.78, maximum);
    let mut width_b = (base * (1.0 - asymmetry)).clamp(minimum * 0.78, maximum);
    if matches!(
        kind,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc
    ) {
        if traits.overriding_plate == traits.plate_a {
            width_a = base.clamp(minimum, maximum);
            width_b = (base * 0.18).clamp(45.0, maximum);
        } else {
            width_b = base.clamp(minimum, maximum);
            width_a = (base * 0.18).clamp(45.0, maximum);
        }
    }

    let orthogonality = 1.0 - clamp01(traits.obliquity_deg / 90.0);
    let core_strength = clamp01(
        (0.22 + maturity * 0.58 + shortening * 0.34)
            * (0.70 + orthogonality * 0.30)
            * (0.34 + taper * 0.66),
    );
    let te_norm = clamp01((traits.mean_te_km - 18.0) / 62.0);
    let plateau_eligibility = if kind == OrogenProvinceKind::CollisionalPlateau {
        clamp01(
            smoothstep((maturity - 0.58) / 0.32)
                * smoothstep((shortening - 0.52) / 0.38)
                * (0.55 + traits.weakness * 0.45)
                * (1.0 - 0.42 * te_norm),
        )
    } else {
        0.0
    };
    (
        width_a,
        width_b,
        maturity,
        shortening,
        core_strength,
        plateau_eligibility,
    )
}

fn build_provinces_and_sources''')

replace_once(orogen,
    '                let (width_a, width_b, maturity, shortening, core_strength) =\n                    source_widths(traits, kind, system_has_endpoints);',
    '                let (\n                    width_a,\n                    width_b,\n                    maturity,\n                    shortening,\n                    core_strength,\n                    plateau_eligibility,\n                ) = source_widths(traits, kind, system_has_endpoints);')
replace_once(orogen,
    '                    core_strength,\n                });',
    '                    core_strength,\n                    plateau_eligibility,\n                });')

replace_regex(orogen,
    r'fn nearest_sources<T: PlanetTopology>\(.*?\n\}\n\nfn rasterize<T: PlanetTopology>',
    '''fn sample_interior_resistance(pre: &PreOrogenicLithosphereModel, sample: usize) -> f64 {
    let strength = f64::from(pre.intrinsic_strength_index[sample]).clamp(0.0, 1.0);
    let weakness = f64::from(pre.intrinsic_weakness_index[sample]).clamp(0.0, 1.0);
    let te = clamp01((f64::from(pre.effective_elastic_thickness_km[sample]) - 8.0) / 78.0);
    let fabric = f64::from(pre.inherited_fabric_strength[sample]).clamp(0.0, 1.0);
    let rift = f64::from(pre.inherited_rift_memory[sample]).clamp(0.0, 1.0);
    let shear = f64::from(pre.inherited_shear_memory[sample]).clamp(0.0, 1.0);
    let age_break = f64::from(pre.age_discontinuity_index[sample]).clamp(0.0, 1.0);
    let province_break = f64::from(pre.province_boundary_index[sample]).clamp(0.0, 1.0);
    let weak_corridor = clamp01(0.42 * fabric + 0.30 * rift + 0.18 * shear + 0.10 * age_break);
    clamp01(
        0.12 + 0.50 * strength + 0.20 * te + 0.18 * (1.0 - weakness)
            - 0.48 * weak_corridor
            - 0.10 * province_break,
    )
}

fn propagation_penalty(kind: OrogenProvinceKind, resistance: f64) -> f64 {
    let r = clamp01(resistance);
    let gain = match kind {
        OrogenProvinceKind::CollisionalPlateau => 1.60,
        OrogenProvinceKind::ContinentalCollision => 2.60,
        OrogenProvinceKind::TerraneAccretion => 2.20,
        OrogenProvinceKind::TranspressionalOrogen => 3.00,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc => 0.75,
    };
    1.0 + gain * r.powf(1.35)
}

fn nearest_sources<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    sources: &[BoundarySource],
    resistance: &[f64],
    parameters: PlanetPhysicalParameters,
) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
    let count = topology.sample_count() as usize;
    let mut cost = vec![f64::INFINITY; count];
    let mut physical_distance = vec![f64::INFINITY; count];
    let mut source_id = vec![usize::MAX; count];
    let mut frontier = BinaryHeap::new();

    for (source_index, source) in sources.iter().enumerate() {
        let boundary = &tectonics.boundaries[source.boundary_index];
        for sample in [boundary.sample_a, boundary.sample_b] {
            let index = sample as usize;
            let plate = tectonics.plate_ids[index];
            if plate != source.plate_a && plate != source.plate_b {
                continue;
            }
            if cost[index] > 0.0 || (cost[index] == 0.0 && source_index < source_id[index]) {
                cost[index] = 0.0;
                physical_distance[index] = 0.0;
                source_id[index] = source_index;
                frontier.push(DistanceFrontier {
                    cost_km: 0.0,
                    physical_distance_km: 0.0,
                    source_index,
                    sample,
                });
            }
        }
    }

    let radius_km = parameters.radius_m / 1000.0;
    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        if current.cost_km > cost[index] + 1.0e-9 || current.source_index != source_id[index] {
            continue;
        }
        let plate = tectonics.plate_ids[index];
        let source = sources[current.source_index];
        let neighbors = topology.neighbors(current.sample);
        let lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let target = neighbor as usize;
            if tectonics.plate_ids[target] != plate {
                continue;
            }
            let step_km = lengths[neighbor_index] * radius_km;
            let edge_resistance = (resistance[index] + resistance[target]) * 0.5;
            let candidate_cost = current.cost_km
                + step_km * propagation_penalty(source.kind, edge_resistance);
            let candidate_physical = current.physical_distance_km + step_km;
            if candidate_cost + 1.0e-9 < cost[target]
                || ((candidate_cost - cost[target]).abs() <= 1.0e-9
                    && current.source_index < source_id[target])
            {
                cost[target] = candidate_cost;
                physical_distance[target] = candidate_physical;
                source_id[target] = current.source_index;
                frontier.push(DistanceFrontier {
                    cost_km: candidate_cost,
                    physical_distance_km: candidate_physical,
                    source_index: current.source_index,
                    sample: neighbor,
                });
            }
        }
    }
    (cost, physical_distance, source_id)
}

fn smooth_within_province<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    province_ids: &[u16],
    values: &[f32],
) -> Vec<f32> {
    let mut smoothed = values.to_vec();
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let province = province_ids[index];
        if province == 0 {
            continue;
        }
        let plate = tectonics.plate_ids[index];
        let mut sum = f64::from(values[index]) * 4.0;
        let mut weight = 4.0;
        for neighbor in topology.neighbors(sample) {
            let neighbor = *neighbor as usize;
            if province_ids[neighbor] == province && tectonics.plate_ids[neighbor] == plate {
                sum += f64::from(values[neighbor]);
                weight += 1.0;
            }
        }
        smoothed[index] = (sum / weight) as f32;
    }
    smoothed
}

fn rasterize<T: PlanetTopology>''')

replace_regex(orogen,
    r'fn rasterize<T: PlanetTopology>\(.*?\n\}\n\nfn model_hash',
    '''fn rasterize<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    pre: &PreOrogenicLithosphereModel,
    sources: &[BoundarySource],
    parameters: PlanetPhysicalParameters,
) -> (
    Vec<u16>,
    Vec<u8>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
) {
    let count = topology.sample_count() as usize;
    let resistance = (0..count)
        .map(|sample| sample_interior_resistance(pre, sample))
        .collect::<Vec<_>>();
    let (cost_distance, physical_distance, source_id) =
        nearest_sources(topology, tectonics, sources, &resistance, parameters);
    let mut province_ids = vec![0_u16; count];
    let mut province_kind = vec![0_u8; count];
    let mut orogenic = vec![0.0_f32; count];
    let mut boundary_distance = vec![0.0_f32; count];
    let mut mountain_core = vec![0.0_f32; count];
    let interior_resistance = resistance.iter().map(|value| *value as f32).collect::<Vec<_>>();
    let mut root = vec![0.0_f32; count];
    let mut plateau = vec![0.0_f32; count];
    let mut fold_thrust = vec![0.0_f32; count];
    let mut foreland = vec![0.0_f32; count];
    let mut arc = vec![0.0_f32; count];
    let mut backarc = vec![0.0_f32; count];
    let mut suture = vec![0.0_f32; count];
    let mut transpression = vec![0.0_f32; count];
    let mut maturity = vec![0.0_f32; count];
    let mut shortening = vec![0.0_f32; count];
    let mut local_width = vec![0.0_f32; count];
    let mut along_fraction = vec![0.0_f32; count];

    for sample in 0..count {
        let source_index = source_id[sample];
        if source_index == usize::MAX
            || !cost_distance[sample].is_finite()
            || !physical_distance[sample].is_finite()
        {
            continue;
        }
        let source = sources[source_index];
        let plate = tectonics.plate_ids[sample];
        let width = if plate == source.plate_a {
            source.width_a_km
        } else if plate == source.plate_b {
            source.width_b_km
        } else {
            continue;
        };
        let x = cost_distance[sample] / width.max(1.0);
        let reach_limit = match source.kind {
            OrogenProvinceKind::CollisionalPlateau => 1.85,
            OrogenProvinceKind::ContinentalCollision => 1.70,
            OrogenProvinceKind::TerraneAccretion => 1.65,
            OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc => 1.60,
            OrogenProvinceKind::TranspressionalOrogen => 1.45,
        };
        if x > reach_limit {
            continue;
        }
        let reach_envelope = smoothstep((reach_limit - x) / (reach_limit * 0.42));
        let overriding = plate == source.overriding_plate;
        let hinterland = plate == source.hinterland_plate;
        let foreland_side = plate == source.foreland_or_subducting_plate;
        let subduction = matches!(
            source.kind,
            OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc
        );
        let resistance_value = resistance[sample];
        let transmission = (1.0 - 0.42 * resistance_value).clamp(0.52, 1.0);
        let distance_km = physical_distance[sample];

        let (
            mountain_value,
            root_value,
            plateau_value,
            fold_value,
            foreland_value,
            arc_value,
            backarc_value,
            suture_value,
            transpression_value,
            intensity,
        ) = if subduction {
            if overriding {
                // The mountain/volcanic arc is explicitly offset inland from the trench. The
                // offset is in physical kilometres so a very broad province cannot push the arc
                // thousands of kilometres into the overriding plate.
                let arc_center_km = 145.0 + 120.0 * source.maturity + 35.0 * source.curvature;
                let arc_sigma_km = 55.0 + 30.0 * (1.0 - resistance_value);
                let arc_profile = gaussian(distance_km, arc_center_km, arc_sigma_km);
                let mountain = clamp01(
                    source.core_strength
                        * (0.58 + 0.42 * source.maturity)
                        * arc_profile
                        * reach_envelope,
                );
                let root_value = clamp01(
                    source.core_strength
                        * gaussian(distance_km, arc_center_km, arc_sigma_km * 1.45)
                        * 0.58
                        * transmission,
                );
                let fold_value = clamp01(
                    source.core_strength
                        * gaussian(distance_km, arc_center_km + 70.0, arc_sigma_km * 1.35)
                        * 0.38,
                );
                let arc_value = clamp01(
                    (0.52 + 0.48 * source.maturity)
                        * gaussian(distance_km, arc_center_km, arc_sigma_km * 0.78),
                );
                let backarc_value = clamp01(
                    source.maturity
                        * gaussian(distance_km, arc_center_km + 220.0, 110.0)
                        * 0.66,
                );
                let suture_value = clamp01(
                    source.core_strength * gaussian(distance_km, 0.0, 70.0) * 0.16,
                );
                let intensity = mountain.max(arc_value * 0.82).max(suture_value * 0.20);
                (
                    mountain,
                    root_value,
                    0.0,
                    fold_value,
                    0.0,
                    arc_value,
                    backarc_value,
                    suture_value,
                    0.0,
                    intensity,
                )
            } else {
                let suture_value = clamp01(
                    source.core_strength * gaussian(distance_km, 0.0, 65.0) * 0.20,
                );
                (
                    0.0,
                    clamp01(suture_value * 0.25),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    suture_value,
                    0.0,
                    suture_value * 0.18,
                )
            }
        } else {
            let side_amplitude = if hinterland {
                1.0
            } else if foreland_side {
                0.92
            } else {
                0.76
            };
            let mountain = clamp01(
                source.core_strength
                    * gaussian(x, 0.10, 0.20)
                    * side_amplitude
                    * (0.82 + 0.18 * transmission)
                    * reach_envelope,
            );
            let root_value = clamp01(
                source.core_strength
                    * gaussian(x, 0.18, 0.28)
                    * if hinterland { 1.0 } else { 0.78 }
                    * transmission,
            );
            let plateau_value = if source.kind == OrogenProvinceKind::CollisionalPlateau {
                clamp01(
                    source.plateau_eligibility
                        * gaussian(x, 0.50, 0.28)
                        * if hinterland { 1.0 } else { 0.22 }
                        * transmission,
                )
            } else {
                0.0
            };
            let fold_value = clamp01(
                source.core_strength
                    * gaussian(x, 0.64, 0.20)
                    * if foreland_side { 1.0 } else { 0.45 }
                    * (0.72 + 0.28 * transmission),
            );
            let foreland_value = if foreland_side {
                clamp01(
                    source.maturity
                        * source.shortening
                        * gaussian(x, 1.04, 0.17)
                        * reach_envelope,
                )
            } else {
                0.0
            };
            let suture_value = clamp01(source.core_strength * gaussian(x, 0.0, 0.08));
            let transpression_value = clamp01(
                source.core_strength
                    * source.obliquity
                    * (0.72 + source.curvature * 0.28)
                    * gaussian(x, 0.10, 0.16),
            );
            let intensity = mountain
                .max(fold_value * 0.75)
                .max(plateau_value * 0.55)
                .max(transpression_value * 0.85)
                * reach_envelope;
            (
                mountain,
                root_value,
                plateau_value,
                fold_value,
                foreland_value,
                0.0,
                0.0,
                suture_value,
                transpression_value,
                intensity,
            )
        };

        let active_signal = intensity
            .max(root_value * 0.45)
            .max(fold_value * 0.50)
            .max(foreland_value * 0.35)
            .max(backarc_value * 0.30)
            .max(suture_value * 0.20);
        if active_signal < 0.025 {
            continue;
        }

        province_ids[sample] = source.province_id;
        province_kind[sample] = source.kind as u8;
        orogenic[sample] = intensity as f32;
        boundary_distance[sample] = distance_km as f32;
        mountain_core[sample] = mountain_value as f32;
        root[sample] = root_value as f32;
        plateau[sample] = plateau_value as f32;
        fold_thrust[sample] = fold_value as f32;
        foreland[sample] = foreland_value as f32;
        arc[sample] = arc_value as f32;
        backarc[sample] = backarc_value as f32;
        suture[sample] = suture_value as f32;
        transpression[sample] = transpression_value as f32;
        maturity[sample] = source.maturity as f32;
        shortening[sample] = source.shortening as f32;
        local_width[sample] = width as f32;
        along_fraction[sample] = source.along_fraction as f32;
    }

    // Remove source-cell/Voronoi seams without blurring across plates or between distinct
    // tectonic provinces. The tectonic geometry stays causal; only adjacent samples owned by the
    // same connected province share a small amount of structural state.
    orogenic = smooth_within_province(topology, tectonics, &province_ids, &orogenic);
    mountain_core = smooth_within_province(topology, tectonics, &province_ids, &mountain_core);
    root = smooth_within_province(topology, tectonics, &province_ids, &root);
    plateau = smooth_within_province(topology, tectonics, &province_ids, &plateau);
    fold_thrust = smooth_within_province(topology, tectonics, &province_ids, &fold_thrust);
    foreland = smooth_within_province(topology, tectonics, &province_ids, &foreland);
    arc = smooth_within_province(topology, tectonics, &province_ids, &arc);
    backarc = smooth_within_province(topology, tectonics, &province_ids, &backarc);
    suture = smooth_within_province(topology, tectonics, &province_ids, &suture);
    transpression = smooth_within_province(topology, tectonics, &province_ids, &transpression);

    (
        province_ids,
        province_kind,
        orogenic,
        boundary_distance,
        mountain_core,
        interior_resistance,
        root,
        plateau,
        fold_thrust,
        foreland,
        arc,
        backarc,
        suture,
        transpression,
        maturity,
        shortening,
        local_width,
        along_fraction,
    )
}

fn model_hash''')

replace_once(orogen,
    '    hash = hash_f32(hash, &model.orogenic_intensity);\n    hash = hash_f32(hash, &model.crustal_root_index);',
    '    hash = hash_f32(hash, &model.orogenic_intensity);\n    hash = hash_f32(hash, &model.boundary_distance_km);\n    hash = hash_f32(hash, &model.mountain_core_index);\n    hash = hash_f32(hash, &model.interior_resistance_index);\n    hash = hash_f32(hash, &model.crustal_root_index);')
replace_once(orogen,
    '        model.orogenic_intensity.len(),\n        model.crustal_root_index.len(),',
    '        model.orogenic_intensity.len(),\n        model.boundary_distance_km.len(),\n        model.mountain_core_index.len(),\n        model.interior_resistance_index.len(),\n        model.crustal_root_index.len(),')
replace_once(orogen,
    '    let normalized: [&[f32]; 11] = [\n        &model.orogenic_intensity,\n        &model.crustal_root_index,',
    '    let normalized: [&[f32]; 13] = [\n        &model.orogenic_intensity,\n        &model.mountain_core_index,\n        &model.interior_resistance_index,\n        &model.crustal_root_index,')
replace_once(orogen,
    '        .any(|value| !value.is_finite() || *value < 0.0 || *value > 2200.001)',
    '        .any(|value| !value.is_finite() || *value < 0.0 || *value > 1000.001)')
replace_once(orogen,
    '            "orogen province width is outside supported bounds",',
    '            "orogen deformation reach is outside supported bounds",')
replace_once(orogen,
    '    for sample in 0..count {\n        let id = model.province_ids[sample];',
    '    if model\n        .boundary_distance_km\n        .iter()\n        .any(|value| !value.is_finite() || *value < 0.0 || *value > 2500.0)\n    {\n        return Err(WorldgenError::InvalidLithosphere(\n            "orogen boundary distance is outside supported bounds",\n        ));\n    }\n    for sample in 0..count {\n        let id = model.province_ids[sample];')

replace_once(orogen,
    '        orogenic_intensity,\n        crustal_root_index,',
    '        orogenic_intensity,\n        boundary_distance_km,\n        mountain_core_index,\n        interior_resistance_index,\n        crustal_root_index,')
replace_once(orogen,
    '    ) = rasterize(topology, tectonics, &sources, parameters);',
    '    ) = rasterize(topology, tectonics, pre, &sources, parameters);')
replace_once(orogen,
    '        orogenic_intensity,\n        crustal_root_index,\n        plateau_index,',
    '        orogenic_intensity,\n        boundary_distance_km,\n        mountain_core_index,\n        interior_resistance_index,\n        crustal_root_index,\n        plateau_index,')

# Add focused architectural invariants.
replace_once(orogen,
    '''    #[test]\n    fn orogen_provinces_reference_real_connected_segments() {''',
    '''    #[test]\n    fn inland_resistance_increases_collision_propagation_cost() {\n        let low = propagation_penalty(OrogenProvinceKind::ContinentalCollision, 0.10);\n        let high = propagation_penalty(OrogenProvinceKind::ContinentalCollision, 0.90);\n        let plateau_high = propagation_penalty(OrogenProvinceKind::CollisionalPlateau, 0.90);\n        assert!(high > low * 2.0);\n        assert!(plateau_high < high);\n    }\n\n    #[test]\n    fn mountain_cores_remain_boundary_localized() {\n        let seed = "wg36-boundary-localized-core";\n        let (topology, tectonics, history, geology, pre) = world(seed);\n        let model = generate_tectonic_orogen_provinces(\n            &topology,\n            &tectonics,\n            &history,\n            &geology,\n            &pre,\n            &OrogenProvinceRequest::new(seed),\n            PlanetPhysicalParameters::earthlike_reference(),\n        )\n        .unwrap();\n        let mut core_count = 0usize;\n        for sample in 0..model.mountain_core_index.len() {\n            if model.mountain_core_index[sample] > 0.20 {\n                core_count += 1;\n                assert!(\n                    model.boundary_distance_km[sample] < 800.0,\n                    "mountain core escaped too far inland: {:.1} km",\n                    model.boundary_distance_km[sample]\n                );\n            }\n        }\n        assert!(core_count > 0);\n    }\n\n    #[test]\n    fn orogen_provinces_reference_real_connected_segments() {''')

causal = 'rust/interlink-worldgen/src/causal_pipeline.rs'
replace_once(causal,
    '    OrogenProvinceModel, OrogenProvinceRequest, PlanetPhysicalParameters, PlanetTopology,',
    '    OrogenProvinceKind, OrogenProvinceModel, OrogenProvinceRequest, PlanetPhysicalParameters, PlanetTopology,')
replace_once(causal,
    'pub const TECTONIC_TOPOGRAPHY_STAGE_VERSION: u32 = 12;\nconst TECTONIC_TOPOGRAPHY_NAMESPACE: &str = "terrain:tectonic-province-topography:v1";',
    'pub const TECTONIC_TOPOGRAPHY_STAGE_VERSION: u32 = 13;\nconst TECTONIC_TOPOGRAPHY_NAMESPACE: &str = "terrain:boundary-localized-orogen-topography:v2";')
replace_once(causal,
    '    pub orogenic_history: Vec<f32>,\n    pub volcanic_arc_history: Vec<f32>,',
    '    pub orogenic_history: Vec<f32>,\n    pub boundary_distance_km: Vec<f32>,\n    pub mountain_core_index: Vec<f32>,\n    pub interior_resistance_index: Vec<f32>,\n    pub volcanic_arc_history: Vec<f32>,')
replace_once(causal,
    '        self.volcanic_arc_history = Vec::new();',
    '        self.boundary_distance_km = Vec::new();\n        self.mountain_core_index = Vec::new();\n        self.interior_resistance_index = Vec::new();\n        self.volcanic_arc_history = Vec::new();')
replace_once(causal,
    '    let orogenic_history = refine(&orogens.orogenic_intensity)?;\n    let volcanic_arc_history = refine(&orogens.volcanic_arc_index)?;',
    '    let orogenic_history = refine(&orogens.orogenic_intensity)?;\n    let boundary_distance_km = refine(&orogens.boundary_distance_km)?;\n    let mountain_core_index = refine(&orogens.mountain_core_index)?;\n    let interior_resistance_index = refine(&orogens.interior_resistance_index)?;\n    let volcanic_arc_history = refine(&orogens.volcanic_arc_index)?;')
replace_once(causal,
    '        orogenic_history,\n        volcanic_arc_history,',
    '        orogenic_history,\n        boundary_distance_km,\n        mountain_core_index,\n        interior_resistance_index,\n        volcanic_arc_history,')

replace_regex(causal,
    r'fn province_relief\(inherited: &InheritedPhysicalState, index: usize\) -> \(f64, f64\) \{.*?\n\}\n\npub fn generate_initial_topography',
    '''fn province_relief(inherited: &InheritedPhysicalState, index: usize) -> (f64, f64) {
    let intensity = f64::from(inherited.orogenic_history[index]).clamp(0.0, 1.0);
    let mountain_core = f64::from(inherited.mountain_core_index[index]).clamp(0.0, 1.0);
    let resistance = f64::from(inherited.interior_resistance_index[index]).clamp(0.0, 1.0);
    let root = f64::from(inherited.crustal_root_index[index]).clamp(0.0, 1.0);
    let plateau = f64::from(inherited.plateau_index[index]).clamp(0.0, 1.0);
    let fold = f64::from(inherited.fold_thrust_index[index]).clamp(0.0, 1.0);
    let foreland = f64::from(inherited.foreland_basin_index[index]).clamp(0.0, 1.0);
    let volcanic_arc = f64::from(inherited.volcanic_arc_history[index]).clamp(0.0, 1.0);
    let backarc = f64::from(inherited.backarc_extension_index[index]).clamp(0.0, 1.0);
    let suture = f64::from(inherited.suture_index[index]).clamp(0.0, 1.0);
    let transpression = f64::from(inherited.transpression_index[index]).clamp(0.0, 1.0);
    let maturity = f64::from(inherited.maturity_index[index]).clamp(0.0, 1.0);
    let shortening = f64::from(inherited.shortening_index[index]).clamp(0.0, 1.0);
    let kind = inherited.province_kind[index];
    let subduction = kind == OrogenProvinceKind::CordilleranArc as u8
        || kind == OrogenProvinceKind::IslandArc as u8;
    let tectonic_gain = 0.82 + 0.18 * maturity + 0.22 * shortening;
    let broad_transmission = 0.72 + 0.28 * (1.0 - resistance);

    if subduction {
        // Subduction topography is one-sided and arc-centred. The collision component is zero so
        // an oceanic/continental margin cannot accidentally receive both a collision mountain and
        // a volcanic arc at the same location.
        let arc_relief = 1_850.0 * mountain_core
            + volcanic_arc * (2_650.0 + 900.0 * maturity)
            + 420.0 * fold
            + 160.0 * intensity
            - 900.0 * backarc
            - 120.0 * suture;
        return (0.0, arc_relief);
    }

    let crust_scale = match inherited.crust_kind[index] {
        CRUST_OCEANIC => 0.18,
        CRUST_TRANSITIONAL => 0.62,
        _ => 1.0,
    };
    let collision_relief = crust_scale
        * tectonic_gain
        * (5_600.0 * mountain_core
            + 2_450.0 * root * broad_transmission
            + 1_650.0 * plateau * broad_transmission
            + 1_900.0 * fold
            + 2_500.0 * transpression
            + 260.0 * intensity
            - 1_350.0 * foreland
            - 180.0 * suture);
    (collision_relief, 0.0)
}

pub fn generate_initial_topography''')
replace_once(causal,
    '        || inherited.crustal_root_index.len() != count\n        || inherited.province_ids.len() != count',
    '        || inherited.crustal_root_index.len() != count\n        || inherited.mountain_core_index.len() != count\n        || inherited.boundary_distance_km.len() != count\n        || inherited.province_ids.len() != count')

smoke = 'rust/interlink-worldgen-cli/examples/tectonic_orogen_provinces_smoke.rs'
replace_once(smoke,
    '    println!(\n        "WG-3.6 tectonic orogens: provinces={} collision={} plateau={} cordilleran={} island_arc={} terrane={} transpressional={} coverage={:.1}% width={:.0}/{:.0}/{:.0}km max_shortening={:.3} max_intensity={:.3} hash={}",',
    '    let mountain_core_samples = orogens.mountain_core_index.iter().filter(|value| **value > 0.20).count();\n    let far_core_samples = orogens\n        .mountain_core_index\n        .iter()\n        .zip(orogens.boundary_distance_km.iter())\n        .filter(|(core, distance)| **core > 0.20 && **distance >= 800.0)\n        .count();\n\n    println!(\n        "WG-3.6 tectonic orogens: provinces={} collision={} plateau={} cordilleran={} island_arc={} terrane={} transpressional={} coverage={:.1}% width={:.0}/{:.0}/{:.0}km core={} far_core={} max_shortening={:.3} max_intensity={:.3} hash={}",')
replace_once(smoke,
    '        orogens.metrics.maximum_source_width_km,\n        orogens.metrics.maximum_shortening_index,',
    '        orogens.metrics.maximum_source_width_km,\n        mountain_core_samples,\n        far_core_samples,\n        orogens.metrics.maximum_shortening_index,')
replace_once(smoke,
    '    if orogens.provinces.is_empty() {',
    '    if mountain_core_samples == 0 || far_core_samples != 0 {\n        return Err("mountain-core localization invariant failed".to_string());\n    }\n    if orogens.provinces.is_empty() {')

top_smoke = 'rust/interlink-worldgen-cli/examples/tectonic_topography_cutover_smoke.rs'
replace_once(top_smoke,
    '    let positive_orogen = terrain',
    '    let mountain_core_samples = inherited\n        .mountain_core_index\n        .iter()\n        .filter(|value| **value > 0.20)\n        .count();\n    let far_mountain_core_samples = inherited\n        .mountain_core_index\n        .iter()\n        .zip(inherited.boundary_distance_km.iter())\n        .filter(|(core, distance)| **core > 0.20 && **distance >= 800.0)\n        .count();\n    let positive_orogen = terrain')
replace_once(top_smoke,
    '        "WG-4 tectonic topography cutover: stage=v{} provinces={} active_samples={} relief(+/-)={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",',
    '        "WG-4 boundary-localized topography: stage=v{} provinces={} active_samples={} core={} far_core={} relief(+/-)={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",')
replace_once(top_smoke,
    '        active_samples,\n        positive_orogen,',
    '        active_samples,\n        mountain_core_samples,\n        far_mountain_core_samples,\n        positive_orogen,')
replace_once(top_smoke,
    '    if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 12 {',
    '    if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 13 {')
replace_once(top_smoke,
    '    if active_samples == 0 || positive_orogen == 0 || negative_orogen == 0 {',
    '    if active_samples == 0\n        || mountain_core_samples == 0\n        || far_mountain_core_samples != 0\n        || positive_orogen == 0\n        || negative_orogen == 0\n    {')

Path('docs/worldgen-rewrite/BOUNDARY_LOCALIZED_OROGENS.md').write_text('''# Boundary-localized orogens (WG-3.6 v2 / WG-4 v13)\n\nThis rewrite makes convergent boundaries the organizing spine of major relief while treating broad continental plateaus as an exceptional collision outcome.\n\n## Mechanical changes\n\n- Province width is a deformation-reach scale, not a direct mountain footprint.\n- Collision propagation is a weighted graph solve. Strong, thick pre-orogenic lithosphere raises travel cost; inherited weak fabric, rifts, shear zones, and crustal discontinuities lower the effective resistance upstream.\n- A separate narrow `mountain_core_index` represents the high-relief range. Crustal root, fold-thrust, foreland, plateau, suture, arc, and back-arc fields remain distinct.\n- Plateau classification now requires mature, strongly shortened, mechanically permissive continental collision. Plateau relief is lower-amplitude than the mountain core and predominantly hinterland-side.\n- Subduction relief is explicitly one-sided: trench/boundary -> inland arc mountain core -> back-arc. Arc offset is expressed in physical kilometres rather than as a fraction of an arbitrarily broad province.\n- Fine-grid structural fields are still scratch state and are released immediately after WG-4, preserving the L8 memory architecture.\n\n## Geometric changes\n\n- Nominal collision reaches are hundreds rather than thousands of kilometres; only exceptional plateau systems can approach ~1000 km source scale.\n- Active deformation is cut off before source footprints can spread across an entire continent.\n- Source/Voronoi seams are locally blended only within the same connected province and plate, so the numerical ownership tessellation does not directly appear as triangular relief.\n- End tapers remain causal and connected to boundary-system topology.\n\n## Acceptance invariants\n\nThe blocking tests require that stronger continental interiors cost more to deform and that samples belonging to the high-relief mountain core remain within 800 km of their connected convergent boundary. These are structural invariants, not legacy morphology calibration targets.\n''')

print('boundary-localized orogen rewrite applied')
