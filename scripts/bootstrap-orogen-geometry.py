from pathlib import Path

path = Path("rust/interlink-worldgen/src/geology.rs")
text = path.read_text()
old_identity = 'pub const GEOLOGY_STAGE_VERSION: u32 = 3;\nconst GEOLOGY_NAMESPACE: &str = "worldgen:geology:crust-history:v3";'
new_identity = 'pub const GEOLOGY_STAGE_VERSION: u32 = 4;\nconst GEOLOGY_NAMESPACE: &str = "worldgen:geology:crust-history:v4";'
assert old_identity in text, "geology stage identity anchor missing"
text = text.replace(old_identity, new_identity, 1)

anchor = "fn influence_field<T: PlanetTopology>(topology: &T, sources: &[u32], scale_rad: f64) -> Vec<f32> {"
assert anchor in text, "influence_field anchor missing"
addition = r'''fn weighted_influence_field<T: PlanetTopology>(
    topology: &T,
    source_strength: &[f64],
    scale_rad: f64,
) -> Vec<f32> {
    let count = topology.sample_count() as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut source_id = vec![u32::MAX; count];
    let mut frontier = BinaryHeap::new();
    for sample in 0..count {
        let strength = source_strength[sample].clamp(0.0, 1.0);
        if strength <= 0.0 {
            continue;
        }
        let initial_distance = -scale_rad * strength.ln();
        distance[sample] = initial_distance;
        source_id[sample] = sample as u32;
        frontier.push(DistanceFrontier {
            distance_rad: initial_distance,
            source: sample as u32,
            sample: sample as u32,
        });
    }
    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        if current.distance_rad > distance[index] + 1.0e-14 || current.source != source_id[index] {
            continue;
        }
        let neighbors = topology.neighbors(current.sample);
        let lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let candidate = current.distance_rad + lengths[neighbor_index];
            let target = neighbor as usize;
            if candidate + 1.0e-14 < distance[target] {
                distance[target] = candidate;
                source_id[target] = current.source;
                frontier.push(DistanceFrontier {
                    distance_rad: candidate,
                    source: current.source,
                    sample: neighbor,
                });
            }
        }
    }
    distance
        .into_iter()
        .map(|value| {
            if value.is_finite() {
                (-value / scale_rad).exp().clamp(0.0, 1.0) as f32
            } else {
                0.0
            }
        })
        .collect()
}

fn orogenic_influence_field<T: PlanetTopology>(
    topology: &T,
    kinds: &[u8],
    ages: &[f32],
    boundaries: &[GeologicalBoundary],
    sources: &[u32],
    base_scale_rad: f64,
) -> Vec<f32> {
    let count = topology.sample_count() as usize;
    if sources.is_empty() {
        return vec![0.0; count];
    }

    let mut source_mask = vec![false; count];
    for source in sources {
        source_mask[*source as usize] = true;
    }
    let mut strength = vec![0.0_f64; count];
    let mut width_control = vec![0.0_f64; count];
    let mut expected_neighbors = vec![0.0_f64; count];

    let mut register = |sample: u32, convergence: f64, collision: bool| {
        let index = sample as usize;
        let age_control = clamp01((f64::from(ages[index]) - 250.0) / 2_250.0);
        let crust_adjustment = match crust_kind(kinds[index]) {
            CrustKind::Continental => 0.06,
            CrustKind::Transitional => -0.04,
            CrustKind::Oceanic => -0.12,
        };
        let raw_width = if collision {
            0.55 * convergence + 0.45 * age_control + crust_adjustment
        } else {
            0.48 * convergence + 0.37 * age_control + 0.03 + crust_adjustment
        };
        strength[index] = 1.0;
        width_control[index] = width_control[index].max(raw_width.clamp(0.0, 1.0));
        expected_neighbors[index] = expected_neighbors[index].max(if collision { 3.0 } else { 2.0 });
    };

    for boundary in boundaries {
        let convergence = clamp01(boundary.normal_rate_m_per_year.abs() / 0.08);
        match boundary.regime {
            GeologicalBoundaryRegime::ContinentalCollision => {
                register(boundary.sample_a, convergence, true);
                register(boundary.sample_b, convergence, true);
            }
            GeologicalBoundaryRegime::OceanicSubduction
            | GeologicalBoundaryRegime::OceanContinentSubduction => match boundary.subduction_polarity {
                SubductionPolarity::PlateA => register(boundary.sample_b, convergence, false),
                SubductionPolarity::PlateB => register(boundary.sample_a, convergence, false),
                SubductionPolarity::None => {}
            },
            _ => {}
        }
    }

    let mut narrow = vec![0.0_f64; count];
    let mut standard = vec![0.0_f64; count];
    let mut broad = vec![0.0_f64; count];
    for sample in 0..count {
        if strength[sample] <= 0.0 {
            continue;
        }
        let neighbor_sources = topology
            .neighbors(sample as u32)
            .iter()
            .filter(|neighbor| source_mask[**neighbor as usize])
            .count() as f64;
        let support = if expected_neighbors[sample] > 0.0 {
            (neighbor_sources / expected_neighbors[sample]).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let control = (0.68 * width_control[sample] + 0.32 * support).clamp(0.0, 1.0);
        if control < 0.43 {
            narrow[sample] = 1.0;
        } else if control < 0.68 {
            standard[sample] = 1.0;
        } else {
            broad[sample] = 1.0;
        }
    }

    let narrow_field = weighted_influence_field(topology, &narrow, base_scale_rad * 0.86);
    let standard_field = weighted_influence_field(topology, &standard, base_scale_rad);
    let broad_field = weighted_influence_field(topology, &broad, base_scale_rad * 1.16);
    let mut field = vec![0.0_f32; count];
    for sample in 0..count {
        field[sample] = f64::from(narrow_field[sample])
            .max(f64::from(standard_field[sample]))
            .max(f64::from(broad_field[sample]))
            .clamp(0.0, 1.0) as f32;
    }
    field
}

'''
text = text.replace(anchor, addition + anchor, 1)

old_call = "    let mut orogen = influence_field(topology, &orogen_sources, orogen_scale);"
new_call = '''    let mut orogen = orogenic_influence_field(
        topology,
        kinds,
        ages,
        boundaries,
        &orogen_sources,
        orogen_scale,
    );'''
assert old_call in text, "orogenic history call anchor missing"
text = text.replace(old_call, new_call, 1)
path.write_text(text)

ci = Path(".github/workflows/ci.yml")
ci_text = ci.read_text()
ci_anchor = "      - name: WG-4 L5→L7 inherited-orogen morphology acceptance\n        run: cargo run --release -p interlink-worldgen-cli --example topography_inherited_orogen_morphology_acceptance\n"
assert ci_anchor in ci_text, "CI anchor missing"
ci_insert = ci_anchor + "      - name: WG-3 inherited-orogen source-geometry acceptance\n        run: cargo run --release -p interlink-worldgen-cli --example geology_orogen_geometry_acceptance\n"
ci.write_text(ci_text.replace(ci_anchor, ci_insert, 1))

docs = Path("docs/worldgen-rewrite/TOPOGRAPHY.md")
docs_text = docs.read_text()
docs_anchor = "WG-4 `@11` reshapes **inherited orogenic relief**"
assert docs_anchor in docs_text, "topography documentation anchor missing"
paragraph_end = docs_text.find("\n\n", docs_text.index(docs_anchor))
assert paragraph_end > 0
note = (
    "\n\nWG-3 geological history stage `v4` diversifies the upstream `orogenic_history` geometry "
    "without reopening accepted continental assembly. Orogenic sources retain deterministic tectonic provenance, "
    "while bounded narrow/standard/broad propagation envelopes are selected from convergence, crustal age/type, "
    "and local boundary-chain support. The source cores remain unity-strength; geometry changes are therefore "
    "expressed primarily through along-strike width variation and termination taper rather than amplitude retuning. "
    "No new terrain-noise source is introduced. WG-4 `@11` continues to own the history-to-relief conversion and "
    "the accepted `400 km` active-collision kernel remains unchanged."
)
docs.write_text(docs_text[:paragraph_end] + note + docs_text[paragraph_end:])
