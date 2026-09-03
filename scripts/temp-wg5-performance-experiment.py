from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''fn mean_ocean_neighbor(
    topology: &GeodesicTopology,
    ocean: &[bool],
    values: &[f64],
    sample: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for neighbor in topology.neighbors_of(sample as u32) {
        let index = *neighbor as usize;
        if ocean[index] {
            sum += values[index];
            count += 1;
        }
    }
    if count > 0 {
        sum / count as f64
    } else {
        values[sample]
    }
}
''',
    '''#[derive(Clone, Debug)]
struct OceanNeighborGeometry {
    offsets: Vec<usize>,
    neighbors: Vec<u32>,
}

fn build_ocean_neighbor_geometry(
    topology: &GeodesicTopology,
    ocean: &[bool],
) -> OceanNeighborGeometry {
    let sample_count = topology.metrics().sample_count as usize;
    let mut offsets = Vec::with_capacity(sample_count + 1);
    let mut neighbors = Vec::with_capacity(sample_count.saturating_mul(6));
    offsets.push(0);
    for sample in 0..sample_count {
        for neighbor in topology.neighbors_of(sample as u32) {
            if ocean[*neighbor as usize] {
                neighbors.push(*neighbor);
            }
        }
        offsets.push(neighbors.len());
    }
    OceanNeighborGeometry { offsets, neighbors }
}

fn mean_ocean_neighbor_cached(
    geometry: &OceanNeighborGeometry,
    values: &[f64],
    sample: usize,
) -> f64 {
    let start = geometry.offsets[sample];
    let end = geometry.offsets[sample + 1];
    let neighbors = &geometry.neighbors[start..end];
    let mut sum = 0.0;
    for neighbor in neighbors {
        sum += values[*neighbor as usize];
    }
    if !neighbors.is_empty() {
        sum / neighbors.len() as f64
    } else {
        values[sample]
    }
}
''',
    'replace ocean neighbor scan with cached geometry',
)

replace_once(
    '''        water_depth_m[i] = f64::from(terrain.water_depth_m[i]).max(0.0);
    }

    let scalar_gradient_geometry =
''',
    '''        water_depth_m[i] = f64::from(terrain.water_depth_m[i]).max(0.0);
    }

    let ocean_neighbor_geometry = build_ocean_neighbor_geometry(topology, &ocean);
    let scalar_gradient_geometry =
''',
    'build ocean neighbor cache once',
)

replace_once(
    '''                let neighbor_sst = mean_ocean_neighbor(topology, &ocean, &previous_sst, i);
''',
    '''                let neighbor_sst =
                    mean_ocean_neighbor_cached(&ocean_neighbor_geometry, &previous_sst, i);
''',
    'use cached ocean neighbors for sst diffusion',
)

path.write_text(text)
