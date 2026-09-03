from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def rep(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} marker not found')
    text = text.replace(old, new, 1)

rep(
'''    pub sea_surface_temperature_mean_k: Vec<f32>,
    pub current_east_mean_m_s: Vec<f32>,
    pub current_north_mean_m_s: Vec<f32>,
''',
'''    pub sea_surface_temperature_mean_k: Vec<f32>,
    pub sea_surface_temperature_annual_cos_k: Vec<f32>,
    pub sea_surface_temperature_annual_sin_k: Vec<f32>,
    pub current_east_mean_m_s: Vec<f32>,
    pub current_north_mean_m_s: Vec<f32>,
    pub current_east_annual_cos_m_s: Vec<f32>,
    pub current_east_annual_sin_m_s: Vec<f32>,
    pub current_north_annual_cos_m_s: Vec<f32>,
    pub current_north_annual_sin_m_s: Vec<f32>,
''', 'state fields')
rep(
'''    let mut sst_sum = vec![0.0; sample_count];
    let mut current_east_sum = vec![0.0; sample_count];
    let mut current_north_sum = vec![0.0; sample_count];
''',
'''    let mut sst_sum = vec![0.0; sample_count];
    let mut sst_cos = vec![0.0; sample_count];
    let mut sst_sin = vec![0.0; sample_count];
    let mut current_east_sum = vec![0.0; sample_count];
    let mut current_north_sum = vec![0.0; sample_count];
    let mut current_east_cos = vec![0.0; sample_count];
    let mut current_east_sin = vec![0.0; sample_count];
    let mut current_north_cos = vec![0.0; sample_count];
    let mut current_north_sin = vec![0.0; sample_count];
''', 'accumulators')
rep(
'''        sst_sum.fill(0.0);
        current_east_sum.fill(0.0);
        current_north_sum.fill(0.0);
''',
'''        sst_sum.fill(0.0);
        sst_cos.fill(0.0);
        sst_sin.fill(0.0);
        current_east_sum.fill(0.0);
        current_north_sum.fill(0.0);
        current_east_cos.fill(0.0);
        current_east_sin.fill(0.0);
        current_north_cos.fill(0.0);
        current_north_sin.fill(0.0);
''', 'accumulator reset')
rep(
'''                sst_sum[i] += sea_surface_temperature[i];
                current_east_sum[i] += current_east[i];
                current_north_sum[i] += current_north[i];
''',
'''                sst_sum[i] += sea_surface_temperature[i];
                sst_cos[i] += sea_surface_temperature[i] * phase_cos;
                sst_sin[i] += sea_surface_temperature[i] * phase_sin;
                current_east_sum[i] += current_east[i];
                current_north_sum[i] += current_north[i];
                current_east_cos[i] += current_east[i] * phase_cos;
                current_east_sin[i] += current_east[i] * phase_sin;
                current_north_cos[i] += current_north[i] * phase_cos;
                current_north_sin[i] += current_north[i] * phase_sin;
''', 'phase accumulation')
rep(
'''    let mut sst_mean = vec![0.0; sample_count];
    let mut current_east_mean = vec![0.0; sample_count];
    let mut current_north_mean = vec![0.0; sample_count];
''',
'''    let mut sst_mean = vec![0.0; sample_count];
    let mut sst_cos_out = vec![0.0; sample_count];
    let mut sst_sin_out = vec![0.0; sample_count];
    let mut current_east_mean = vec![0.0; sample_count];
    let mut current_north_mean = vec![0.0; sample_count];
    let mut current_east_cos_out = vec![0.0; sample_count];
    let mut current_east_sin_out = vec![0.0; sample_count];
    let mut current_north_cos_out = vec![0.0; sample_count];
    let mut current_north_sin_out = vec![0.0; sample_count];
''', 'output buffers')
rep(
'''        sst_mean[i] = (sst_sum[i] / phase_count_f64) as f32;
        current_east_mean[i] = (current_east_sum[i] / phase_count_f64) as f32;
        current_north_mean[i] = (current_north_sum[i] / phase_count_f64) as f32;
''',
'''        sst_mean[i] = (sst_sum[i] / phase_count_f64) as f32;
        sst_cos_out[i] = (sst_cos[i] * harmonic_scale) as f32;
        sst_sin_out[i] = (sst_sin[i] * harmonic_scale) as f32;
        current_east_mean[i] = (current_east_sum[i] / phase_count_f64) as f32;
        current_north_mean[i] = (current_north_sum[i] / phase_count_f64) as f32;
        current_east_cos_out[i] = (current_east_cos[i] * harmonic_scale) as f32;
        current_east_sin_out[i] = (current_east_sin[i] * harmonic_scale) as f32;
        current_north_cos_out[i] = (current_north_cos[i] * harmonic_scale) as f32;
        current_north_sin_out[i] = (current_north_sin[i] * harmonic_scale) as f32;
''', 'output harmonic assembly')
rep(
'''    climate_hash = hash_f32_slice(climate_hash, &sst_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_east_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_north_mean);
''',
'''    climate_hash = hash_f32_slice(climate_hash, &sst_mean);
    climate_hash = hash_f32_slice(climate_hash, &sst_cos_out);
    climate_hash = hash_f32_slice(climate_hash, &sst_sin_out);
    climate_hash = hash_f32_slice(climate_hash, &current_east_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_north_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_east_cos_out);
    climate_hash = hash_f32_slice(climate_hash, &current_east_sin_out);
    climate_hash = hash_f32_slice(climate_hash, &current_north_cos_out);
    climate_hash = hash_f32_slice(climate_hash, &current_north_sin_out);
''', 'hash harmonics')
rep(
'''        sea_surface_temperature_mean_k: sst_mean,
        current_east_mean_m_s: current_east_mean,
        current_north_mean_m_s: current_north_mean,
''',
'''        sea_surface_temperature_mean_k: sst_mean,
        sea_surface_temperature_annual_cos_k: sst_cos_out,
        sea_surface_temperature_annual_sin_k: sst_sin_out,
        current_east_mean_m_s: current_east_mean,
        current_north_mean_m_s: current_north_mean,
        current_east_annual_cos_m_s: current_east_cos_out,
        current_east_annual_sin_m_s: current_east_sin_out,
        current_north_annual_cos_m_s: current_north_cos_out,
        current_north_annual_sin_m_s: current_north_sin_out,
''', 'state return')
path.write_text(text)

# Extend the WASM climate bridge.
path = Path('rust/interlink-worldgen-wasm/src/climate_bridge.rs')
text = path.read_text()
old = '''    pub fn sea_surface_temperature_mean_k(&self) -> Vec<f32> { self.climate.sea_surface_temperature_mean_k.clone() }
    pub fn current_east_mean_m_s(&self) -> Vec<f32> { self.climate.current_east_mean_m_s.clone() }
    pub fn current_north_mean_m_s(&self) -> Vec<f32> { self.climate.current_north_mean_m_s.clone() }
'''
new = '''    pub fn sea_surface_temperature_mean_k(&self) -> Vec<f32> { self.climate.sea_surface_temperature_mean_k.clone() }
    pub fn sea_surface_temperature_annual_cos_k(&self) -> Vec<f32> { self.climate.sea_surface_temperature_annual_cos_k.clone() }
    pub fn sea_surface_temperature_annual_sin_k(&self) -> Vec<f32> { self.climate.sea_surface_temperature_annual_sin_k.clone() }
    pub fn current_east_mean_m_s(&self) -> Vec<f32> { self.climate.current_east_mean_m_s.clone() }
    pub fn current_north_mean_m_s(&self) -> Vec<f32> { self.climate.current_north_mean_m_s.clone() }
    pub fn current_east_annual_cos_m_s(&self) -> Vec<f32> { self.climate.current_east_annual_cos_m_s.clone() }
    pub fn current_east_annual_sin_m_s(&self) -> Vec<f32> { self.climate.current_east_annual_sin_m_s.clone() }
    pub fn current_north_annual_cos_m_s(&self) -> Vec<f32> { self.climate.current_north_annual_cos_m_s.clone() }
    pub fn current_north_annual_sin_m_s(&self) -> Vec<f32> { self.climate.current_north_annual_sin_m_s.clone() }
'''
if old not in text:
    raise SystemExit('climate bridge seasonal marker not found')
path.write_text(text.replace(old, new, 1))
