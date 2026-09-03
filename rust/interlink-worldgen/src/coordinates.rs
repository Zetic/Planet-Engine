use crate::WorldgenError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TangentBasis {
    pub east: [f64; 3],
    pub north: [f64; 3],
    pub up: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceAnchor {
    pub direction: [f64; 3],
    pub planet_radius_m: f64,
    pub altitude_m: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalEnuPosition {
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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
fn normalize(value: [f64; 3]) -> Result<[f64; 3], WorldgenError> {
    let magnitude = norm(value);
    if !magnitude.is_finite() || magnitude <= f64::EPSILON {
        return Err(WorldgenError::InvalidCoordinate(
            "direction must be finite and non-zero",
        ));
    }
    Ok([
        value[0] / magnitude,
        value[1] / magnitude,
        value[2] / magnitude,
    ])
}

pub fn unit_vector_from_lat_lon_degrees(
    latitude_deg: f64,
    longitude_deg: f64,
) -> Result<[f64; 3], WorldgenError> {
    if !latitude_deg.is_finite()
        || !longitude_deg.is_finite()
        || !(-90.0..=90.0).contains(&latitude_deg)
    {
        return Err(WorldgenError::InvalidCoordinate(
            "latitude/longitude must be finite and latitude must be within ±90 degrees",
        ));
    }
    let latitude = latitude_deg.to_radians();
    let longitude = longitude_deg.to_radians();
    let cos_latitude = latitude.cos();
    Ok([
        cos_latitude * longitude.cos(),
        cos_latitude * longitude.sin(),
        latitude.sin(),
    ])
}

pub fn lat_lon_degrees_from_unit_vector(direction: [f64; 3]) -> Result<(f64, f64), WorldgenError> {
    let unit = normalize(direction)?;
    Ok((
        unit[2].clamp(-1.0, 1.0).asin().to_degrees(),
        unit[1].atan2(unit[0]).to_degrees(),
    ))
}

pub fn tangent_basis(direction: [f64; 3]) -> Result<TangentBasis, WorldgenError> {
    let up = normalize(direction)?;
    let (_, longitude_deg) = lat_lon_degrees_from_unit_vector(up)?;
    let longitude = longitude_deg.to_radians();
    let east = [-longitude.sin(), longitude.cos(), 0.0];
    let north = cross(up, east);
    Ok(TangentBasis { east, north, up })
}

pub fn great_circle_distance_m(
    a: [f64; 3],
    b: [f64; 3],
    planet_radius_m: f64,
) -> Result<f64, WorldgenError> {
    if !planet_radius_m.is_finite() || planet_radius_m <= 0.0 {
        return Err(WorldgenError::InvalidCoordinate(
            "planet radius must be finite and positive",
        ));
    }
    let first = normalize(a)?;
    let second = normalize(b)?;
    Ok(dot(first, second).clamp(-1.0, 1.0).acos() * planet_radius_m)
}

pub fn anchor_origin_cartesian(anchor: SurfaceAnchor) -> Result<[f64; 3], WorldgenError> {
    if !anchor.planet_radius_m.is_finite()
        || anchor.planet_radius_m <= 0.0
        || !anchor.altitude_m.is_finite()
    {
        return Err(WorldgenError::InvalidCoordinate(
            "surface anchor radius/altitude must be finite and radius must be positive",
        ));
    }
    let direction = normalize(anchor.direction)?;
    let radius = anchor.planet_radius_m + anchor.altitude_m;
    if radius <= 0.0 {
        return Err(WorldgenError::InvalidCoordinate(
            "surface anchor altitude places the anchor at or below the planet center",
        ));
    }
    Ok([
        direction[0] * radius,
        direction[1] * radius,
        direction[2] * radius,
    ])
}

pub fn local_enu_to_cartesian(
    anchor: SurfaceAnchor,
    local: LocalEnuPosition,
) -> Result<[f64; 3], WorldgenError> {
    if !local.east_m.is_finite() || !local.north_m.is_finite() || !local.up_m.is_finite() {
        return Err(WorldgenError::InvalidCoordinate(
            "local ENU coordinates must be finite",
        ));
    }
    let basis = tangent_basis(anchor.direction)?;
    let origin = anchor_origin_cartesian(anchor)?;
    Ok([
        origin[0]
            + basis.east[0] * local.east_m
            + basis.north[0] * local.north_m
            + basis.up[0] * local.up_m,
        origin[1]
            + basis.east[1] * local.east_m
            + basis.north[1] * local.north_m
            + basis.up[1] * local.up_m,
        origin[2]
            + basis.east[2] * local.east_m
            + basis.north[2] * local.north_m
            + basis.up[2] * local.up_m,
    ])
}

pub fn cartesian_to_local_enu(
    anchor: SurfaceAnchor,
    cartesian: [f64; 3],
) -> Result<LocalEnuPosition, WorldgenError> {
    if cartesian.iter().any(|value| !value.is_finite()) {
        return Err(WorldgenError::InvalidCoordinate(
            "Cartesian coordinates must be finite",
        ));
    }
    let basis = tangent_basis(anchor.direction)?;
    let origin = anchor_origin_cartesian(anchor)?;
    let delta = [
        cartesian[0] - origin[0],
        cartesian[1] - origin[1],
        cartesian[2] - origin[2],
    ];
    Ok(LocalEnuPosition {
        east_m: dot(delta, basis.east),
        north_m: dot(delta, basis.north),
        up_m: dot(delta, basis.up),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geodetic_round_trip_is_stable_away_from_pole_longitude_degeneracy() {
        for (latitude, longitude) in [
            (-75.0, -170.0),
            (-20.0, 45.0),
            (0.0, 0.0),
            (32.5, -110.75),
            (80.0, 179.0),
        ] {
            let direction = unit_vector_from_lat_lon_degrees(latitude, longitude).unwrap();
            let (round_latitude, round_longitude) =
                lat_lon_degrees_from_unit_vector(direction).unwrap();
            assert!((latitude - round_latitude).abs() < 1.0e-10);
            assert!((longitude - round_longitude).abs() < 1.0e-10);
        }
    }
    #[test]
    fn tangent_basis_is_orthonormal_even_at_the_poles() {
        for direction in [
            unit_vector_from_lat_lon_degrees(0.0, 0.0).unwrap(),
            unit_vector_from_lat_lon_degrees(90.0, 0.0).unwrap(),
            unit_vector_from_lat_lon_degrees(-90.0, 135.0).unwrap(),
        ] {
            let basis = tangent_basis(direction).unwrap();
            assert!(dot(basis.east, basis.north).abs() < 1.0e-12);
            assert!(dot(basis.east, basis.up).abs() < 1.0e-12);
            assert!(dot(basis.north, basis.up).abs() < 1.0e-12);
            assert!((norm(basis.east) - 1.0).abs() < 1.0e-12);
            assert!((norm(basis.north) - 1.0).abs() < 1.0e-12);
            assert!((norm(basis.up) - 1.0).abs() < 1.0e-12);
        }
    }
    #[test]
    fn meter_scale_enu_coordinates_round_trip_on_an_earth_sized_globe() {
        let anchor = SurfaceAnchor {
            direction: unit_vector_from_lat_lon_degrees(32.0, -110.0).unwrap(),
            planet_radius_m: 6_371_000.0,
            altitude_m: 850.0,
        };
        let local = LocalEnuPosition {
            east_m: 34.25,
            north_m: -12.5,
            up_m: 3.0,
        };
        let world = local_enu_to_cartesian(anchor, local).unwrap();
        let recovered = cartesian_to_local_enu(anchor, world).unwrap();
        assert!((local.east_m - recovered.east_m).abs() < 1.0e-8);
        assert!((local.north_m - recovered.north_m).abs() < 1.0e-8);
        assert!((local.up_m - recovered.up_m).abs() < 1.0e-8);
    }
    #[test]
    fn great_circle_distance_matches_quarter_circumference() {
        let a = unit_vector_from_lat_lon_degrees(0.0, 0.0).unwrap();
        let b = unit_vector_from_lat_lon_degrees(0.0, 90.0).unwrap();
        let radius = 6_371_000.0;
        let distance = great_circle_distance_m(a, b, radius).unwrap();
        assert!((distance - std::f64::consts::FRAC_PI_2 * radius).abs() < 1.0e-6);
    }
}
