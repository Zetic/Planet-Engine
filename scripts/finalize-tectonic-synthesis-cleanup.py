from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    assert count == 1, f"{label}: expected 1 occurrence, found {count}"
    return text.replace(old, new, 1)


# WG-3.6 must classify convergent sources from the same geological regime/polarity
# authority used by WG-3. Otherwise transitional-margin subduction can be counted by
# geology while being silently reinterpreted as collision by the orogen rasterizer.
orogen = Path('rust/interlink-worldgen/src/orogen_provinces.rs')
text = orogen.read_text()
text = replace_once(
    text,
    '    random, CrustKind, CrustalModel, InheritedStructureKind, PlanetPhysicalParameters,\n    PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind, PreOrogenicLithosphereModel,\n    StageIdentity, TectonicHistoryModel, TectonicModel, WorldgenError,',
    '    random, CrustKind, CrustalModel, GeologicalBoundaryRegime, InheritedStructureKind,\n    PlanetPhysicalParameters, PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind,\n    PreOrogenicLithosphereModel, StageIdentity, SubductionPolarity, TectonicHistoryModel,\n    TectonicModel, WorldgenError,',
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
        SubductionPolarity::None => {
            let buoyancy_a = geology.buoyancy_index[boundary.sample_a as usize];
            let buoyancy_b = geology.buoyancy_index[boundary.sample_b as usize];
            if buoyancy_a <= buoyancy_b {
                boundary.plate_a
            } else {
                boundary.plate_b
            }
        }
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
# Oblique subduction is still subduction: preserve arc morphology and let obliquity
# contribute transpressional structure without reclassifying the whole source as a
# collision-style transpressional orogen.
text = replace_once(
    text,
    '    if mean_obliquity >= 62.0 {\n        return OrogenProvinceKind::TranspressionalOrogen;\n    }',
    '    if mean_obliquity >= 62.0 && first.class == BoundaryClass::Collision {\n        return OrogenProvinceKind::TranspressionalOrogen;\n    }',
    'preserve subduction arc class under high obliquity',
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
