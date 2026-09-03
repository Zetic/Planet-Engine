from pathlib import Path

path = Path('rust/interlink-worldgen-cli/src/main.rs')
text = path.read_text()
old = '''    let climate = generate_coupled_climate(
        &fine,
        &terrain,
        planet,
        &ClimateRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let metrics = &climate.metrics;
'''
new = '''    let climate_request = ClimateRequest::new(options.seed.as_str());
    let climate = generate_coupled_climate(
        &fine,
        &terrain,
        planet,
        &climate_request,
    )
    .map_err(|error| error.to_string())?;
    let metrics = &climate.metrics;
'''
assert old in text
text = text.replace(old, new, 1)
old = '''    println!("elapsed_ms={:.3}", started.elapsed().as_secs_f64() * 1_000.0);
    Ok(())
}

fn profile(_options: &Options) -> Result<(), String> {
'''
new = '''    println!("elapsed_ms={:.3}", started.elapsed().as_secs_f64() * 1_000.0);
    if metrics.final_temperature_rms_change_k
        > climate_request.parameters.convergence_temperature_rms_k
    {
        return Err(format!(
            "WG-5 climate did not converge: final RMS change {:.6} K exceeds target {:.6} K after {} model years",
            metrics.final_temperature_rms_change_k,
            climate_request.parameters.convergence_temperature_rms_k,
            metrics.spinup_years
        ));
    }
    if metrics.moisture_budget_relative_error > 1.0e-8 {
        return Err(format!(
            "WG-5 moisture budget did not close: relative error {:.6e} exceeds diagnostic tolerance 1e-8",
            metrics.moisture_budget_relative_error
        ));
    }
    Ok(())
}

fn profile(_options: &Options) -> Result<(), String> {
'''
assert old in text
path.write_text(text.replace(old, new, 1))
