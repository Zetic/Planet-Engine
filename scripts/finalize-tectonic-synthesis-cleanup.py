from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    assert count == 1, f"{label}: expected 1 occurrence, found {count}"
    return text.replace(old, new, 1)


# Continental material is an area-budgeted ancestral assembly, not a Bernoulli decision per
# ancestral plate. This keeps continental coverage stable across seeds while preserving a
# deterministic spatially irregular carrier set.
historical = Path('rust/interlink-worldgen/src/historical_lithosphere.rs')
text = historical.read_text()
old_carriers = '''    let count = ancestral.plates.len();
    let mut carriers = (0..count)
        .map(|plate| unit_random(seed ^ (plate as u64).wrapping_mul(0xa076_1d64_78bd_642f)) > 0.55)
        .collect::<Vec<_>>();
    if !carriers.iter().any(|carrier| *carrier) {
        if let Some((largest, _)) = ancestral
            .plates
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.area_steradians.total_cmp(&right.area_steradians))
        {
            carriers[largest] = true;
        }
    }
'''
new_carriers = '''    let count = ancestral.plates.len();
    let total_area = ancestral
        .plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>()
        .max(1.0e-12);
    let target_fraction = 0.50
        + (unit_random(seed ^ 0x3c79_ac49_2ba7_b653) - 0.5) * 0.08;
    let target_area = total_area * target_fraction.clamp(0.46, 0.54);
    let mut ranked = (0..count)
        .map(|plate| {
            (
                unit_random(seed ^ (plate as u64).wrapping_mul(0xa076_1d64_78bd_642f)),
                plate,
            )
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|(score_a, plate_a), (score_b, plate_b)| {
        score_b
            .total_cmp(score_a)
            .then_with(|| plate_a.cmp(plate_b))
    });
    let mut carriers = vec![false; count];
    let mut carrier_area = 0.0_f64;
    for (_, plate) in ranked {
        carriers[plate] = true;
        carrier_area += ancestral.plates[plate].area_steradians;
        if carrier_area >= target_area {
            break;
        }
    }
'''
text = replace_once(text, old_carriers, new_carriers, 'area-budget continental carriers')

old_depth = '''    let assembly_count = plate_assemblies
        .iter()
        .copied()
        .max()
        .map(|value| value as usize + 1)
        .unwrap_or(0);
    let mut max_depth = vec![0_u16; assembly_count];
    for sample in 0..count {
        if sample_carrier[sample] && depth[sample] != u16::MAX {
            let assembly = sample_assembly[sample] as usize;
            max_depth[assembly] = max_depth[assembly].max(depth[sample]);
        }
    }
'''
new_depth = '''    // Convert the coarse mesh scale into a bounded physical continental-margin width. The
    // outer carrier ring is transitional crust; oceanic crust begins in the neighboring
    // non-carrier material domain rather than being painted as an automatic moat inside every
    // continental assembly.
    let mut edge_length_sum_km = 0.0_f64;
    let mut edge_length_count = 0_u64;
    for sample in 0..topology.sample_count() {
        let position = topology.unit_position(sample);
        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            edge_length_sum_km += arc_radians(position, topology.unit_position(*neighbor))
                * planet.radius_m
                / 1000.0;
            edge_length_count += 1;
        }
    }
    let mean_edge_km = if edge_length_count > 0 {
        edge_length_sum_km / edge_length_count as f64
    } else {
        350.0
    };
    let transition_steps = (360.0 / mean_edge_km.max(1.0)).ceil().clamp(1.0, 3.0) as u16;
'''
text = replace_once(text, old_depth, new_depth, 'physical transitional margin width')

old_kind = '''        let kind = if sample_carrier[sample_index] {
            let assembly = sample_assembly[sample_index] as usize;
            let maximum = max_depth[assembly].max(1);
            let inward = if depth[sample_index] == u16::MAX {
                1.0
            } else {
                f64::from(depth[sample_index]) / f64::from(maximum)
            };
            let continental_cut = (0.18 + (edge_warp - 0.5) * 0.08).clamp(0.12, 0.24);
            let transitional_cut = (0.045 + (edge_warp - 0.5) * 0.035).clamp(0.02, 0.08);
            if inward >= continental_cut {
                CrustKind::Continental
            } else if inward >= transitional_cut {
                CrustKind::Transitional
            } else {
                CrustKind::Oceanic
            }
        } else {
            CrustKind::Oceanic
        };
'''
new_kind = '''        let kind = if sample_carrier[sample_index] {
            let local_transition_steps = transition_steps
                + if edge_warp > 0.72 && transition_steps < 3 { 1 } else { 0 };
            if depth[sample_index] == u16::MAX || depth[sample_index] < local_transition_steps {
                CrustKind::Transitional
            } else {
                CrustKind::Continental
            }
        } else {
            CrustKind::Oceanic
        };
'''
text = replace_once(text, old_kind, new_kind, 'assembled continental edge material')

old_component_assert = '''            assert!(
                largest_fraction >= 0.06,
                "seed {seed} lacked a substantial assembled continent"
            );
'''
new_component_assert = '''            assert!(
                largest_fraction >= 0.06,
                "seed {seed} lacked a substantial assembled continent"
            );
            assert!(
                (0.20..=0.50).contains(&model.metrics.continental_area_fraction),
                "seed {seed} continental coverage escaped the synthesis envelope: {}",
                model.metrics.continental_area_fraction
            );
            assert!(
                (0.02..=0.18).contains(&model.metrics.transitional_area_fraction),
                "seed {seed} transitional margins collapsed or dominated: {}",
                model.metrics.transitional_area_fraction
            );
            assert!(
                (0.38..=0.75).contains(&model.metrics.oceanic_area_fraction),
                "seed {seed} ocean coverage escaped the synthesis envelope: {}",
                model.metrics.oceanic_area_fraction
            );
'''
text = replace_once(text, old_component_assert, new_component_assert, 'continental coverage regression envelope')
historical.write_text(text)


# Geology owns the material meaning of an active boundary. A convergent contact without an
# oceanic side is collision, including transitional/continental contacts; it must not fall
# through to the subduction counters merely because one side is transitional.
frontend = Path('rust/interlink-worldgen/src/historical_frontend.rs')
text = frontend.read_text()
old_fallback = '''            _ => {
                let polarity = if buoyancy[edge.sample_a as usize]
                    <= buoyancy[edge.sample_b as usize]
                {
                    SubductionPolarity::PlateA
                } else {
                    SubductionPolarity::PlateB
                };
                (GeologicalBoundaryRegime::OceanContinentSubduction, polarity)
            }
'''
new_fallback = '''            _ => (
                GeologicalBoundaryRegime::ContinentalCollision,
                SubductionPolarity::None,
            ),
'''
text = replace_once(text, old_fallback, new_fallback, 'non-oceanic convergence classification')
frontend.write_text(text)


# WG-3.6 consumes the same geological regime/polarity authority used by WG-3. This removes
# duplicate crust-pair interpretation and guarantees subduction metrics and arc morphology
# describe the same active contacts.
orogen = Path('rust/interlink-worldgen/src/orogen_provinces.rs')
text = orogen.read_text()
text = replace_once(
    text,
    '    random, CrustKind, CrustalModel, InheritedStructureKind, PlanetPhysicalParameters,\n    PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind, PreOrogenicLithosphereModel,\n    StageIdentity, TectonicHistoryModel, TectonicModel, WorldgenError,',
    '    random, CrustalModel, GeologicalBoundaryRegime, InheritedStructureKind,\n    PlanetPhysicalParameters, PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind,\n    PreOrogenicLithosphereModel, StageIdentity, SubductionPolarity, TectonicHistoryModel,\n    TectonicModel, WorldgenError,',
    'orogen geological authority imports',
)
old_edge_class = '''fn edge_class(
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
    boundary: &PlateBoundaryEdge,
) -> (BoundaryClass, u16, u16, u16) {
    let kind_a = geology.crust_kind[boundary.sample_a as usize];
    let kind_b = geology.crust_kind[boundary.sample_b as usize];
    let oceanic_a = kind_a == CrustKind::Oceanic as u8;
    let oceanic_b = kind_b == CrustKind::Oceanic as u8;

    if oceanic_a && oceanic_b {
        let age_a = geology.crust_age_myr[boundary.sample_a as usize];
        let age_b = geology.crust_age_myr[boundary.sample_b as usize];
        let subducting = if age_a > age_b {
            boundary.plate_a
        } else if age_b > age_a {
            boundary.plate_b
        } else {
            boundary.plate_a.min(boundary.plate_b)
        };
        let overriding = if subducting == boundary.plate_a {
            boundary.plate_b
        } else {
            boundary.plate_a
        };
        return (BoundaryClass::IslandArc, overriding, NO_PLATE, subducting);
    }

    if oceanic_a != oceanic_b {
        let subducting = if oceanic_a {
            boundary.plate_a
        } else {
            boundary.plate_b
        };
        let overriding = if oceanic_a {
            boundary.plate_b
        } else {
            boundary.plate_a
        };
        return (BoundaryClass::Cordilleran, overriding, NO_PLATE, subducting);
    }

    let strength_a = f64::from(pre.intrinsic_strength_index[boundary.sample_a as usize]);
    let strength_b = f64::from(pre.intrinsic_strength_index[boundary.sample_b as usize]);
    let hinterland = if strength_a > strength_b {
        boundary.plate_a
    } else if strength_b > strength_a {
        boundary.plate_b
    } else {
        boundary.plate_a.min(boundary.plate_b)
    };
    let foreland = if hinterland == boundary.plate_a {
        boundary.plate_b
    } else {
        boundary.plate_a
    };
    (BoundaryClass::Collision, NO_PLATE, hinterland, foreland)
}
'''
new_edge_class = '''fn edge_class(
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
    boundary_index: usize,
    boundary: &PlateBoundaryEdge,
) -> (BoundaryClass, u16, u16, u16) {
    let geological = &geology.boundaries[boundary_index];
    let subducting_from_polarity = || match geological.subduction_polarity {
        SubductionPolarity::PlateA => boundary.plate_a,
        SubductionPolarity::PlateB => boundary.plate_b,
        SubductionPolarity::None => boundary.plate_a.min(boundary.plate_b),
    };

    match geological.regime {
        GeologicalBoundaryRegime::OceanicSubduction => {
            let subducting = subducting_from_polarity();
            let overriding = if subducting == boundary.plate_a {
                boundary.plate_b
            } else {
                boundary.plate_a
            };
            (BoundaryClass::IslandArc, overriding, NO_PLATE, subducting)
        }
        GeologicalBoundaryRegime::OceanContinentSubduction => {
            let subducting = subducting_from_polarity();
            let overriding = if subducting == boundary.plate_a {
                boundary.plate_b
            } else {
                boundary.plate_a
            };
            (BoundaryClass::Cordilleran, overriding, NO_PLATE, subducting)
        }
        _ => {
            let strength_a = f64::from(pre.intrinsic_strength_index[boundary.sample_a as usize]);
            let strength_b = f64::from(pre.intrinsic_strength_index[boundary.sample_b as usize]);
            let hinterland = if strength_a > strength_b {
                boundary.plate_a
            } else if strength_b > strength_a {
                boundary.plate_b
            } else {
                boundary.plate_a.min(boundary.plate_b)
            };
            let foreland = if hinterland == boundary.plate_a {
                boundary.plate_b
            } else {
                boundary.plate_a
            };
            (BoundaryClass::Collision, NO_PLATE, hinterland, foreland)
        }
    }
}
'''
text = replace_once(text, old_edge_class, new_edge_class, 'unify convergent regime classification')
text = replace_once(
    text,
    '            edge_class(geology, pre, boundary);',
    '            edge_class(geology, pre, index, boundary);',
    'pass boundary index to geological authority',
)
# Oblique subduction is still subduction. Keep the arc province and let obliquity affect its
# geometry instead of silently converting the whole source into a collision-style province.
text = replace_once(
    text,
    '    if mean_obliquity >= 62.0 {\n        return OrogenProvinceKind::TranspressionalOrogen;\n    }',
    '    if mean_obliquity >= 62.0 && first.class == BoundaryClass::Collision {\n        return OrogenProvinceKind::TranspressionalOrogen;\n    }',
    'preserve subduction arc class under high obliquity',
)
# Arc centers are physically offset from trenches by less than the spacing of some coarse
# WG-3.6 samples. Antialias the magmatic envelope without moving the source or lowering the
# acceptance threshold.
text = replace_once(
    text,
    'gaussian(physical_distance_km, arc_center_km, arc_sigma_km * 0.78)',
    'gaussian(physical_distance_km, arc_center_km, arc_sigma_km * 1.35)',
    'coarse-grid volcanic arc antialiasing',
)
orogen.write_text(text)


# Physical output and the cumulative browser packet both change in this PR.
ci = Path('.github/workflows/ci.yml')
text = ci.read_text()
text = replace_once(
    text,
    "assert p['run']['engine_version']==16",
    "assert p['run']['engine_version']==17",
    'downstream engine gate',
)
lineage = '''      - name: Verify historical lithosphere lineage and projection\n        run: cargo run --release -p interlink-worldgen-cli --example historical_lithosphere_acceptance\n'''
lineage_with_synthesis = lineage + '''      - name: Verify modern plate topology and continental assembly\n        run: cargo test -p interlink-worldgen historical_lithosphere\n'''
text = replace_once(text, lineage, lineage_with_synthesis, 'tectonic synthesis blocking gate')
ci.write_text(text)
