use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, inherit_historical_identity,
    HistoricalLithosphereRequest, PlanetPhysicalParameters, PlanetTopology,
};
use std::time::Instant;

const MIB: usize = 1024 * 1024;
const MAX_COARSE_HISTORY_MS: u128 = 8_000;
const MAX_FINE_INHERITANCE_MS: u128 = 12_000;
const MAX_FINE_IDENTITY_BYTES: usize = 12 * MIB;
const MAX_COARSE_DENSE_IDENTITY_BYTES: usize = 512 * 1024;

fn main() -> Result<(), String> {
    let seed = "historical-lithosphere-performance";
    let coarse_level = 5;
    let fine_level = 8;
    let plate_count = 16;
    let planet = PlanetPhysicalParameters::earthlike_reference();

    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;

    let history_started = Instant::now();
    let frontend = generate_historical_frontend(
        &coarse,
        &HistoricalLithosphereRequest::new(seed, plate_count),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let history_ms = history_started.elapsed().as_millis();

    let history = &frontend.historical;
    let coarse_samples = coarse.sample_count() as usize;
    let coarse_dense_identity_bytes = history.origin_plate_ids.len() * size_of::<u16>()
        + history.fragment_ids.len() * size_of::<u16>()
        + history.current_plate_ids.len() * size_of::<u16>()
        + history.crust_kind.len() * size_of::<u8>()
        + history.crust_birth_age_myr.len() * size_of::<f32>();
    if coarse_dense_identity_bytes > MAX_COARSE_DENSE_IDENTITY_BYTES {
        return Err(format!(
            "coarse historical material retained {} bytes for {} samples; budget is {}",
            coarse_dense_identity_bytes, coarse_samples, MAX_COARSE_DENSE_IDENTITY_BYTES
        ));
    }
    if history_ms > MAX_COARSE_HISTORY_MS {
        return Err(format!(
            "L{coarse_level} historical frontend took {history_ms}ms; gate is {MAX_COARSE_HISTORY_MS}ms"
        ));
    }

    let inheritance_started = Instant::now();
    let inherited = inherit_historical_identity(&fine, coarse_level, history)
        .map_err(|error| error.to_string())?;
    let inheritance_ms = inheritance_started.elapsed().as_millis();
    let fine_identity_bytes = inherited.map.nearest_coarse_source.len() * size_of::<u32>()
        + inherited.map.inherited_sample_mask.len() * size_of::<u8>()
        + inherited.origin_plate_ids.len() * size_of::<u16>()
        + inherited.fragment_ids.len() * size_of::<u16>()
        + inherited.current_plate_ids.len() * size_of::<u16>()
        + inherited.crust_kind.len() * size_of::<u8>()
        + inherited.crust_birth_age_myr.len() * size_of::<f32>();
    if fine_identity_bytes > MAX_FINE_IDENTITY_BYTES {
        return Err(format!(
            "L{fine_level} persistent historical identity retained {} bytes; budget is {}",
            fine_identity_bytes, MAX_FINE_IDENTITY_BYTES
        ));
    }
    if inheritance_ms > MAX_FINE_INHERITANCE_MS {
        return Err(format!(
            "L{coarse_level}->L{fine_level} historical inheritance took {inheritance_ms}ms; gate is {MAX_FINE_INHERITANCE_MS}ms"
        ));
    }
    if inherited.origin_plate_ids.len() != fine.sample_count() as usize
        || inherited.fragment_ids.len() != fine.sample_count() as usize
        || inherited.current_plate_ids.len() != fine.sample_count() as usize
    {
        return Err("L8 historical identity does not cover the fine topology".to_owned());
    }

    println!(
        "historical-performance coarse=L{} samples={} history={}ms dense={}KiB fragments={} events={} fine=L{} samples={} inheritance={}ms persistent={:.2}MiB hash={}",
        coarse_level,
        coarse.sample_count(),
        history_ms,
        coarse_dense_identity_bytes / 1024,
        history.metrics.fragment_count,
        history.metrics.event_count,
        fine_level,
        fine.sample_count(),
        inheritance_ms,
        fine_identity_bytes as f64 / MIB as f64,
        inherited.identity_hash_hex(),
    );
    Ok(())
}
