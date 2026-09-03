use crate::WorldgenError;
use std::collections::BTreeMap;

pub const MAX_TOPOLOGY_LEVEL: u8 = 8;
pub const INVALID_SAMPLE_ID: u32 = u32::MAX;

/// Narrow physical-topology contract for future world-generation stages.
/// Algorithms consume geometry and adjacency rather than icosphere construction details.
pub trait PlanetTopology {
    fn sample_count(&self) -> u32;
    fn unit_position(&self, sample: u32) -> [f64; 3];
    fn area_steradians(&self, sample: u32) -> f64;
    fn neighbors(&self, sample: u32) -> &[u32];
    fn neighbor_arc_lengths_rad(&self, sample: u32) -> &[f64];
    fn neighbor_interface_arc_lengths_rad(&self, sample: u32) -> &[f64];
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopologyMetrics {
    pub sample_count: u32,
    pub edge_count: u32,
    pub face_count: u32,
    pub five_neighbor_count: u32,
    pub six_neighbor_count: u32,
    pub total_area_steradians: f64,
    pub minimum_area_steradians: f64,
    pub maximum_area_steradians: f64,
    pub mean_area_steradians: f64,
    pub area_coefficient_of_variation: f64,
    pub minimum_edge_arc_radians: f64,
    pub maximum_edge_arc_radians: f64,
    pub mean_edge_arc_radians: f64,
    pub edge_coefficient_of_variation: f64,
    pub minimum_interface_arc_radians: f64,
    pub maximum_interface_arc_radians: f64,
    pub mean_interface_arc_radians: f64,
    pub interface_coefficient_of_variation: f64,
    pub topology_hash: u64,
}
impl TopologyMetrics {
    pub fn topology_hash_hex(&self) -> String {
        format!("{:016x}", self.topology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeodesicTopology {
    level: u8,
    positions: Vec<[f64; 3]>,
    faces: Vec<[u32; 3]>,
    neighbor_offsets: Vec<u32>,
    neighbors: Vec<u32>,
    neighbor_arc_lengths_rad: Vec<f64>,
    neighbor_interface_arc_lengths_rad: Vec<f64>,
    dual_area_steradians: Vec<f64>,
    birth_levels: Vec<u8>,
    parent_edges: Vec<[u32; 2]>,
    metrics: TopologyMetrics,
}

impl GeodesicTopology {
    pub fn level(&self) -> u8 {
        self.level
    }
    pub fn positions(&self) -> &[[f64; 3]] {
        &self.positions
    }
    pub fn faces(&self) -> &[[u32; 3]] {
        &self.faces
    }
    pub fn neighbor_offsets(&self) -> &[u32] {
        &self.neighbor_offsets
    }
    pub fn neighbor_indices(&self) -> &[u32] {
        &self.neighbors
    }
    pub fn neighbor_center_arc_lengths_rad_values(&self) -> &[f64] {
        &self.neighbor_arc_lengths_rad
    }
    pub fn neighbor_interface_arc_lengths_rad_values(&self) -> &[f64] {
        &self.neighbor_interface_arc_lengths_rad
    }
    pub fn dual_area_steradians(&self) -> &[f64] {
        &self.dual_area_steradians
    }
    pub fn birth_levels(&self) -> &[u8] {
        &self.birth_levels
    }
    pub fn parent_edges(&self) -> &[[u32; 2]] {
        &self.parent_edges
    }
    pub fn metrics(&self) -> &TopologyMetrics {
        &self.metrics
    }
    pub fn neighbors_of(&self, sample: u32) -> &[u32] {
        let index = sample as usize;
        let start = self.neighbor_offsets[index] as usize;
        let end = self.neighbor_offsets[index + 1] as usize;
        &self.neighbors[start..end]
    }
    pub fn neighbor_arc_lengths_of(&self, sample: u32) -> &[f64] {
        let index = sample as usize;
        let start = self.neighbor_offsets[index] as usize;
        let end = self.neighbor_offsets[index + 1] as usize;
        &self.neighbor_arc_lengths_rad[start..end]
    }
    pub fn neighbor_interface_arc_lengths_of(&self, sample: u32) -> &[f64] {
        let index = sample as usize;
        let start = self.neighbor_offsets[index] as usize;
        let end = self.neighbor_offsets[index + 1] as usize;
        &self.neighbor_interface_arc_lengths_rad[start..end]
    }
    pub fn flattened_positions(&self) -> Vec<f64> {
        let mut output = Vec::with_capacity(self.positions.len() * 3);
        for position in &self.positions {
            output.extend_from_slice(position);
        }
        output
    }
    pub fn flattened_faces(&self) -> Vec<u32> {
        let mut output = Vec::with_capacity(self.faces.len() * 3);
        for face in &self.faces {
            output.extend_from_slice(face);
        }
        output
    }
    pub fn flattened_parent_edges(&self) -> Vec<u32> {
        let mut output = Vec::with_capacity(self.parent_edges.len() * 2);
        for edge in &self.parent_edges {
            output.extend_from_slice(edge);
        }
        output
    }
}

impl PlanetTopology for GeodesicTopology {
    fn sample_count(&self) -> u32 {
        self.metrics.sample_count
    }
    fn unit_position(&self, sample: u32) -> [f64; 3] {
        self.positions[sample as usize]
    }
    fn area_steradians(&self, sample: u32) -> f64 {
        self.dual_area_steradians[sample as usize]
    }
    fn neighbors(&self, sample: u32) -> &[u32] {
        self.neighbors_of(sample)
    }
    fn neighbor_arc_lengths_rad(&self, sample: u32) -> &[f64] {
        self.neighbor_arc_lengths_of(sample)
    }
    fn neighbor_interface_arc_lengths_rad(&self, sample: u32) -> &[f64] {
        self.neighbor_interface_arc_lengths_of(sample)
    }
}

#[derive(Clone, Copy, Debug)]
struct EdgeRecord {
    a: u32,
    b: u32,
    first_face: u32,
    second_face: u32,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(value: [f64; 3], factor: f64) -> [f64; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(value: [f64; 3]) -> f64 {
    dot(value, value).sqrt()
}
fn normalize(value: [f64; 3]) -> [f64; 3] {
    scale(value, 1.0 / norm(value))
}
fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}
fn canonical_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn regular_icosahedron() -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let positions = vec![
        normalize([-1.0, phi, 0.0]),
        normalize([1.0, phi, 0.0]),
        normalize([-1.0, -phi, 0.0]),
        normalize([1.0, -phi, 0.0]),
        normalize([0.0, -1.0, phi]),
        normalize([0.0, 1.0, phi]),
        normalize([0.0, -1.0, -phi]),
        normalize([0.0, 1.0, -phi]),
        normalize([phi, 0.0, -1.0]),
        normalize([phi, 0.0, 1.0]),
        normalize([-phi, 0.0, -1.0]),
        normalize([-phi, 0.0, 1.0]),
    ];
    let faces = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    (positions, faces)
}

fn midpoint_index(
    a: u32,
    b: u32,
    level: u8,
    positions: &mut Vec<[f64; 3]>,
    birth_levels: &mut Vec<u8>,
    parent_edges: &mut Vec<[u32; 2]>,
    midpoint_cache: &mut BTreeMap<(u32, u32), u32>,
) -> u32 {
    let edge = canonical_edge(a, b);
    if let Some(existing) = midpoint_cache.get(&edge) {
        return *existing;
    }
    let position = normalize(add(positions[a as usize], positions[b as usize]));
    let id = positions.len() as u32;
    positions.push(position);
    birth_levels.push(level);
    parent_edges.push([edge.0, edge.1]);
    midpoint_cache.insert(edge, id);
    id
}

fn build_edge_records(faces: &[[u32; 3]]) -> Result<Vec<EdgeRecord>, WorldgenError> {
    let mut edge_faces: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (face_index, [a, b, c]) in faces.iter().enumerate() {
        for edge in [
            canonical_edge(*a, *b),
            canonical_edge(*b, *c),
            canonical_edge(*c, *a),
        ] {
            edge_faces.entry(edge).or_default().push(face_index as u32);
        }
    }
    let mut records = Vec::with_capacity(edge_faces.len());
    for ((a, b), incident) in edge_faces {
        if incident.len() != 2 {
            return Err(WorldgenError::InvalidTopology(
                "canonical sphere contains a non-manifold edge",
            ));
        }
        records.push(EdgeRecord {
            a,
            b,
            first_face: incident[0],
            second_face: incident[1],
        });
    }
    Ok(records)
}

fn spherical_triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let determinant = dot(a, cross(b, c)).abs();
    let denominator = 1.0 + dot(a, b) + dot(b, c) + dot(c, a);
    (2.0 * determinant.atan2(denominator)).abs()
}
fn spherical_circumcenter(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let mut center = normalize(cross(sub(b, a), sub(c, a)));
    if dot(center, add(add(a, b), c)) < 0.0 {
        center = scale(center, -1.0);
    }
    center
}
fn build_circumcenters(positions: &[[f64; 3]], faces: &[[u32; 3]]) -> Vec<[f64; 3]> {
    faces
        .iter()
        .map(|[a, b, c]| {
            spherical_circumcenter(
                positions[*a as usize],
                positions[*b as usize],
                positions[*c as usize],
            )
        })
        .collect()
}
fn tangent_sort_basis(up: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let reference = if up[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let east = normalize(cross(reference, up));
    let north = cross(up, east);
    (east, north)
}

fn build_dual_areas(
    positions: &[[f64; 3]],
    faces: &[[u32; 3]],
    circumcenters: &[[f64; 3]],
) -> Vec<f64> {
    let mut incidence_degree = vec![0_u32; positions.len()];
    for [a, b, c] in faces {
        incidence_degree[*a as usize] += 1;
        incidence_degree[*b as usize] += 1;
        incidence_degree[*c as usize] += 1;
    }
    let mut offsets = vec![0_u32; positions.len() + 1];
    for index in 0..positions.len() {
        offsets[index + 1] = offsets[index] + incidence_degree[index];
    }
    let mut incident_faces = vec![0_u32; offsets[positions.len()] as usize];
    let mut cursor = offsets[..positions.len()].to_vec();
    for (face_index, [a, b, c]) in faces.iter().enumerate() {
        for sample in [*a, *b, *c] {
            incident_faces[cursor[sample as usize] as usize] = face_index as u32;
            cursor[sample as usize] += 1;
        }
    }
    let mut areas = vec![0.0; positions.len()];
    for sample in 0..positions.len() {
        let up = positions[sample];
        let (east, north) = tangent_sort_basis(up);
        let start = offsets[sample] as usize;
        let end = offsets[sample + 1] as usize;
        let mut ring = incident_faces[start..end].to_vec();
        ring.sort_by(|left, right| {
            let a = circumcenters[*left as usize];
            let b = circumcenters[*right as usize];
            dot(a, north)
                .atan2(dot(a, east))
                .total_cmp(&dot(b, north).atan2(dot(b, east)))
        });
        let mut area = 0.0;
        for index in 0..ring.len() {
            let first = circumcenters[ring[index] as usize];
            let second = circumcenters[ring[(index + 1) % ring.len()] as usize];
            area += spherical_triangle_area(up, first, second);
        }
        areas[sample] = area;
    }
    areas
}

fn build_neighbor_csr(
    sample_count: usize,
    edges: &[EdgeRecord],
    positions: &[[f64; 3]],
    circumcenters: &[[f64; 3]],
) -> (Vec<u32>, Vec<u32>, Vec<f64>, Vec<f64>) {
    let mut degree = vec![0_u32; sample_count];
    let mut interface_by_edge = BTreeMap::new();
    for edge in edges {
        degree[edge.a as usize] += 1;
        degree[edge.b as usize] += 1;
        interface_by_edge.insert(
            (edge.a, edge.b),
            arc_radians(
                circumcenters[edge.first_face as usize],
                circumcenters[edge.second_face as usize],
            ),
        );
    }
    let mut offsets = vec![0_u32; sample_count + 1];
    for index in 0..sample_count {
        offsets[index + 1] = offsets[index] + degree[index];
    }
    let mut neighbors = vec![0_u32; offsets[sample_count] as usize];
    let mut cursor = offsets[..sample_count].to_vec();
    for edge in edges {
        neighbors[cursor[edge.a as usize] as usize] = edge.b;
        cursor[edge.a as usize] += 1;
        neighbors[cursor[edge.b as usize] as usize] = edge.a;
        cursor[edge.b as usize] += 1;
    }
    for sample in 0..sample_count {
        let start = offsets[sample] as usize;
        let end = offsets[sample + 1] as usize;
        neighbors[start..end].sort_unstable();
    }
    let mut arc_lengths = Vec::with_capacity(neighbors.len());
    let mut interface_lengths = Vec::with_capacity(neighbors.len());
    for sample in 0..sample_count {
        let position = positions[sample];
        for &neighbor in &neighbors[offsets[sample] as usize..offsets[sample + 1] as usize] {
            arc_lengths.push(arc_radians(position, positions[neighbor as usize]));
            interface_lengths.push(
                *interface_by_edge
                    .get(&canonical_edge(sample as u32, neighbor))
                    .expect("closed topology edge missing dual interface"),
            );
        }
    }
    (offsets, neighbors, arc_lengths, interface_lengths)
}

fn mean_and_cv(values: &[f64]) -> (f64, f64) {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    (mean, variance.sqrt() / mean)
}
fn fnv1a_update(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}
fn topology_hash(
    level: u8,
    positions: &[[f64; 3]],
    faces: &[[u32; 3]],
    birth_levels: &[u8],
    parent_edges: &[[u32; 2]],
) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    fnv1a_update(&mut hash, &[level]);
    for position in positions {
        for component in position {
            fnv1a_update(&mut hash, &component.to_bits().to_le_bytes());
        }
    }
    for face in faces {
        for sample in face {
            fnv1a_update(&mut hash, &sample.to_le_bytes());
        }
    }
    fnv1a_update(&mut hash, birth_levels);
    for edge in parent_edges {
        for sample in edge {
            fnv1a_update(&mut hash, &sample.to_le_bytes());
        }
    }
    hash
}

fn build_metrics(
    level: u8,
    positions: &[[f64; 3]],
    faces: &[[u32; 3]],
    edges: &[EdgeRecord],
    neighbor_offsets: &[u32],
    areas: &[f64],
    circumcenters: &[[f64; 3]],
    birth_levels: &[u8],
    parent_edges: &[[u32; 2]],
) -> TopologyMetrics {
    let mut five_neighbor_count = 0_u32;
    let mut six_neighbor_count = 0_u32;
    for sample in 0..positions.len() {
        match neighbor_offsets[sample + 1] - neighbor_offsets[sample] {
            5 => five_neighbor_count += 1,
            6 => six_neighbor_count += 1,
            _ => {}
        }
    }
    let total_area_steradians = areas.iter().sum::<f64>();
    let minimum_area_steradians = areas.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum_area_steradians = areas.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (mean_area_steradians, area_coefficient_of_variation) = mean_and_cv(areas);
    let edge_lengths: Vec<f64> = edges
        .iter()
        .map(|edge| arc_radians(positions[edge.a as usize], positions[edge.b as usize]))
        .collect();
    let minimum_edge_arc_radians = edge_lengths.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum_edge_arc_radians = edge_lengths
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let (mean_edge_arc_radians, edge_coefficient_of_variation) = mean_and_cv(&edge_lengths);
    let interface_lengths: Vec<f64> = edges
        .iter()
        .map(|edge| {
            arc_radians(
                circumcenters[edge.first_face as usize],
                circumcenters[edge.second_face as usize],
            )
        })
        .collect();
    let minimum_interface_arc_radians = interface_lengths
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let maximum_interface_arc_radians = interface_lengths
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let (mean_interface_arc_radians, interface_coefficient_of_variation) =
        mean_and_cv(&interface_lengths);
    TopologyMetrics {
        sample_count: positions.len() as u32,
        edge_count: edges.len() as u32,
        face_count: faces.len() as u32,
        five_neighbor_count,
        six_neighbor_count,
        total_area_steradians,
        minimum_area_steradians,
        maximum_area_steradians,
        mean_area_steradians,
        area_coefficient_of_variation,
        minimum_edge_arc_radians,
        maximum_edge_arc_radians,
        mean_edge_arc_radians,
        edge_coefficient_of_variation,
        minimum_interface_arc_radians,
        maximum_interface_arc_radians,
        mean_interface_arc_radians,
        interface_coefficient_of_variation,
        topology_hash: topology_hash(level, positions, faces, birth_levels, parent_edges),
    }
}

pub fn build_icosphere(level: u8) -> Result<GeodesicTopology, WorldgenError> {
    if level > MAX_TOPOLOGY_LEVEL {
        return Err(WorldgenError::InvalidTopology(
            "icosphere level exceeds the supported WG-1 limit",
        ));
    }
    let (mut positions, mut faces) = regular_icosahedron();
    let mut birth_levels = vec![0_u8; positions.len()];
    let mut parent_edges = vec![[INVALID_SAMPLE_ID, INVALID_SAMPLE_ID]; positions.len()];
    for refinement_level in 1..=level {
        let mut midpoint_cache = BTreeMap::new();
        let mut next_faces = Vec::with_capacity(faces.len() * 4);
        for [a, b, c] in faces.into_iter() {
            let ab = midpoint_index(
                a,
                b,
                refinement_level,
                &mut positions,
                &mut birth_levels,
                &mut parent_edges,
                &mut midpoint_cache,
            );
            let bc = midpoint_index(
                b,
                c,
                refinement_level,
                &mut positions,
                &mut birth_levels,
                &mut parent_edges,
                &mut midpoint_cache,
            );
            let ca = midpoint_index(
                c,
                a,
                refinement_level,
                &mut positions,
                &mut birth_levels,
                &mut parent_edges,
                &mut midpoint_cache,
            );
            next_faces.push([a, ab, ca]);
            next_faces.push([b, bc, ab]);
            next_faces.push([c, ca, bc]);
            next_faces.push([ab, bc, ca]);
        }
        faces = next_faces;
    }
    let edges = build_edge_records(&faces)?;
    let circumcenters = build_circumcenters(&positions, &faces);
    let (neighbor_offsets, neighbors, neighbor_arc_lengths_rad, neighbor_interface_arc_lengths_rad) =
        build_neighbor_csr(positions.len(), &edges, &positions, &circumcenters);
    let dual_area_steradians = build_dual_areas(&positions, &faces, &circumcenters);
    let metrics = build_metrics(
        level,
        &positions,
        &faces,
        &edges,
        &neighbor_offsets,
        &dual_area_steradians,
        &circumcenters,
        &birth_levels,
        &parent_edges,
    );
    Ok(GeodesicTopology {
        level,
        positions,
        faces,
        neighbor_offsets,
        neighbors,
        neighbor_arc_lengths_rad,
        neighbor_interface_arc_lengths_rad,
        dual_area_steradians,
        birth_levels,
        parent_edges,
        metrics,
    })
}

pub fn expected_sample_count(level: u8) -> Option<u32> {
    if level > MAX_TOPOLOGY_LEVEL {
        None
    } else {
        Some(10 * 4_u32.pow(level as u32) + 2)
    }
}
pub fn expected_edge_count(level: u8) -> Option<u32> {
    if level > MAX_TOPOLOGY_LEVEL {
        None
    } else {
        Some(30 * 4_u32.pow(level as u32))
    }
}
pub fn expected_face_count(level: u8) -> Option<u32> {
    if level > MAX_TOPOLOGY_LEVEL {
        None
    } else {
        Some(20 * 4_u32.pow(level as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    #[test]
    fn hierarchy_obeys_closed_icosphere_counts_and_euler_characteristic() {
        for level in 0..=5 {
            let topology = build_icosphere(level).unwrap();
            let metrics = topology.metrics();
            assert_eq!(metrics.sample_count, expected_sample_count(level).unwrap());
            assert_eq!(metrics.edge_count, expected_edge_count(level).unwrap());
            assert_eq!(metrics.face_count, expected_face_count(level).unwrap());
            assert_eq!(
                i64::from(metrics.sample_count) - i64::from(metrics.edge_count)
                    + i64::from(metrics.face_count),
                2
            );
        }
    }
    #[test]
    fn exactly_twelve_sites_are_pentavalent_and_every_other_site_is_hexavalent() {
        for level in 0..=5 {
            let topology = build_icosphere(level).unwrap();
            let metrics = topology.metrics();
            assert_eq!(metrics.five_neighbor_count, 12);
            assert_eq!(metrics.six_neighbor_count, metrics.sample_count - 12);
            for sample in 0..metrics.sample_count {
                let count = topology.neighbors_of(sample).len();
                assert!(count == 5 || count == 6);
                for neighbor in topology.neighbors_of(sample) {
                    assert!(topology.neighbors_of(*neighbor).contains(&sample));
                }
            }
        }
    }
    #[test]
    fn unit_positions_and_dual_areas_cover_the_sphere() {
        for level in 0..=4 {
            let topology = build_icosphere(level).unwrap();
            for position in topology.positions() {
                assert!((norm(*position) - 1.0).abs() < 1.0e-12);
            }
            assert!((topology.metrics().total_area_steradians - 4.0 * PI).abs() < 1.0e-10);
            assert!(topology.metrics().area_coefficient_of_variation < 0.10);
        }
    }
    #[test]
    fn finite_volume_neighbor_geometry_is_positive_symmetric_and_aligned() {
        let topology = build_icosphere(4).unwrap();
        assert_eq!(
            topology.neighbor_indices().len(),
            topology.neighbor_center_arc_lengths_rad_values().len()
        );
        assert_eq!(
            topology.neighbor_indices().len(),
            topology.neighbor_interface_arc_lengths_rad_values().len()
        );
        assert!(topology.metrics().interface_coefficient_of_variation < 0.25);
        for sample in 0..topology.metrics().sample_count {
            let neighbors = topology.neighbors_of(sample);
            let center_lengths = topology.neighbor_arc_lengths_of(sample);
            let interface_lengths = topology.neighbor_interface_arc_lengths_of(sample);
            for index in 0..neighbors.len() {
                assert!(center_lengths[index] > 0.0 && interface_lengths[index] > 0.0);
                let neighbor = neighbors[index];
                let reverse_index = topology
                    .neighbors_of(neighbor)
                    .iter()
                    .position(|candidate| *candidate == sample)
                    .unwrap();
                assert!(
                    (center_lengths[index]
                        - topology.neighbor_arc_lengths_of(neighbor)[reverse_index])
                        .abs()
                        < 1.0e-14
                );
                assert!(
                    (interface_lengths[index]
                        - topology.neighbor_interface_arc_lengths_of(neighbor)[reverse_index])
                        .abs()
                        < 1.0e-14
                );
            }
        }
    }
    #[test]
    fn physical_algorithms_can_consume_the_topology_contract_without_icosphere_internals() {
        fn area_sum<T: PlanetTopology>(topology: &T) -> f64 {
            (0..topology.sample_count())
                .map(|sample| topology.area_steradians(sample))
                .sum()
        }
        let topology = build_icosphere(3).unwrap();
        assert!((area_sum(&topology) - 4.0 * PI).abs() < 1.0e-10);
        let sample = 20;
        assert_eq!(
            topology.neighbors(sample).len(),
            topology.neighbor_arc_lengths_rad(sample).len()
        );
        assert_eq!(
            topology.neighbors(sample).len(),
            topology.neighbor_interface_arc_lengths_rad(sample).len()
        );
        assert!((norm(topology.unit_position(sample)) - 1.0).abs() < 1.0e-12);
    }
    #[test]
    fn refinement_preserves_all_existing_sample_ids_and_provenance() {
        let coarse = build_icosphere(3).unwrap();
        let fine = build_icosphere(4).unwrap();
        assert_eq!(
            &fine.positions()[..coarse.positions().len()],
            coarse.positions()
        );
        assert_eq!(
            &fine.birth_levels()[..coarse.birth_levels().len()],
            coarse.birth_levels()
        );
        assert_eq!(
            &fine.parent_edges()[..coarse.parent_edges().len()],
            coarse.parent_edges()
        );
        for (sample, (&birth, parents)) in fine
            .birth_levels()
            .iter()
            .zip(fine.parent_edges())
            .enumerate()
        {
            if birth == 0 {
                assert_eq!(*parents, [INVALID_SAMPLE_ID, INVALID_SAMPLE_ID]);
            } else {
                assert!(parents[0] < sample as u32 && parents[1] < sample as u32);
            }
        }
    }
    #[test]
    fn topology_hash_is_deterministic_and_level_sensitive() {
        let first = build_icosphere(4).unwrap();
        let second = build_icosphere(4).unwrap();
        let changed = build_icosphere(5).unwrap();
        assert_eq!(
            first.metrics().topology_hash,
            second.metrics().topology_hash
        );
        assert_ne!(
            first.metrics().topology_hash,
            changed.metrics().topology_hash
        );
        assert_eq!(first.metrics().topology_hash_hex().len(), 16);
    }
}
