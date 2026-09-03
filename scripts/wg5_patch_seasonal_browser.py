from pathlib import Path

# Protocol fields.
path = Path('src/worldgen/protocol.ts')
text = path.read_text()
old = '''  seaSurfaceTemperatureMeanK: Float32Array;
  currentEastMeanMS: Float32Array;
  currentNorthMeanMS: Float32Array;
  currentSpeedMeanMS: Float32Array;
'''
new = '''  seaSurfaceTemperatureMeanK: Float32Array;
  seaSurfaceTemperatureAnnualCosK: Float32Array;
  seaSurfaceTemperatureAnnualSinK: Float32Array;
  currentEastMeanMS: Float32Array;
  currentNorthMeanMS: Float32Array;
  currentEastAnnualCosMS: Float32Array;
  currentEastAnnualSinMS: Float32Array;
  currentNorthAnnualCosMS: Float32Array;
  currentNorthAnnualSinMS: Float32Array;
  currentSpeedMeanMS: Float32Array;
'''
if old not in text:
    raise SystemExit('protocol seasonal ocean marker not found')
path.write_text(text.replace(old, new, 1))

# Worker WASM interface and extraction/transport.
path = Path('src/worldgen/worldgenWorker.ts')
text = path.read_text()
old = '''  annual_mean_insolation_w_m2(): Float32Array; seasonal_insolation_amplitude_w_m2(): Float32Array; temperature_mean_k(): Float32Array; temperature_annual_cos_k(): Float32Array; temperature_annual_sin_k(): Float32Array; temperature_min_k(): Float32Array; temperature_max_k(): Float32Array; local_pressure_pa(): Float32Array; wind_east_mean_m_s(): Float32Array; wind_north_mean_m_s(): Float32Array; wind_east_annual_cos_m_s(): Float32Array; wind_east_annual_sin_m_s(): Float32Array; wind_north_annual_cos_m_s(): Float32Array; wind_north_annual_sin_m_s(): Float32Array; sea_surface_temperature_mean_k(): Float32Array; current_east_mean_m_s(): Float32Array; current_north_mean_m_s(): Float32Array; current_speed_mean_m_s(): Float32Array; ocean_heat_transport_index(): Float32Array; specific_humidity_mean(): Float32Array; annual_precipitation_mm(): Float32Array; precipitation_seasonality(): Float32Array; potential_evaporation_mm(): Float32Array; moisture_balance_mm(): Float32Array; aridity_index(): Float32Array; snowfall_fraction(): Float32Array; persistent_snow_potential(): Float32Array; sea_ice_potential(): Float32Array;
'''
new = '''  annual_mean_insolation_w_m2(): Float32Array; seasonal_insolation_amplitude_w_m2(): Float32Array; temperature_mean_k(): Float32Array; temperature_annual_cos_k(): Float32Array; temperature_annual_sin_k(): Float32Array; temperature_min_k(): Float32Array; temperature_max_k(): Float32Array; local_pressure_pa(): Float32Array; wind_east_mean_m_s(): Float32Array; wind_north_mean_m_s(): Float32Array; wind_east_annual_cos_m_s(): Float32Array; wind_east_annual_sin_m_s(): Float32Array; wind_north_annual_cos_m_s(): Float32Array; wind_north_annual_sin_m_s(): Float32Array; sea_surface_temperature_mean_k(): Float32Array; sea_surface_temperature_annual_cos_k(): Float32Array; sea_surface_temperature_annual_sin_k(): Float32Array; current_east_mean_m_s(): Float32Array; current_north_mean_m_s(): Float32Array; current_east_annual_cos_m_s(): Float32Array; current_east_annual_sin_m_s(): Float32Array; current_north_annual_cos_m_s(): Float32Array; current_north_annual_sin_m_s(): Float32Array; current_speed_mean_m_s(): Float32Array; ocean_heat_transport_index(): Float32Array; specific_humidity_mean(): Float32Array; annual_precipitation_mm(): Float32Array; precipitation_seasonality(): Float32Array; potential_evaporation_mm(): Float32Array; moisture_balance_mm(): Float32Array; aridity_index(): Float32Array; snowfall_fraction(): Float32Array; persistent_snow_potential(): Float32Array; sea_ice_potential(): Float32Array;
'''
if old not in text:
    raise SystemExit('worker seasonal interface marker not found')
text = text.replace(old, new, 1)
old = '''const seaSurfaceTemperatureMeanK = output.sea_surface_temperature_mean_k(); const currentEastMeanMS = output.current_east_mean_m_s(); const currentNorthMeanMS = output.current_north_mean_m_s(); const currentSpeedMeanMS = output.current_speed_mean_m_s();'''
new = '''const seaSurfaceTemperatureMeanK = output.sea_surface_temperature_mean_k(); const seaSurfaceTemperatureAnnualCosK = output.sea_surface_temperature_annual_cos_k(); const seaSurfaceTemperatureAnnualSinK = output.sea_surface_temperature_annual_sin_k(); const currentEastMeanMS = output.current_east_mean_m_s(); const currentNorthMeanMS = output.current_north_mean_m_s(); const currentEastAnnualCosMS = output.current_east_annual_cos_m_s(); const currentEastAnnualSinMS = output.current_east_annual_sin_m_s(); const currentNorthAnnualCosMS = output.current_north_annual_cos_m_s(); const currentNorthAnnualSinMS = output.current_north_annual_sin_m_s(); const currentSpeedMeanMS = output.current_speed_mean_m_s();'''
if old not in text:
    raise SystemExit('worker seasonal extraction marker not found')
text = text.replace(old, new, 1)
old = '''seaSurfaceTemperatureMeanK, currentEastMeanMS, currentNorthMeanMS, currentSpeedMeanMS, oceanHeatTransportIndex,'''
new = '''seaSurfaceTemperatureMeanK, seaSurfaceTemperatureAnnualCosK, seaSurfaceTemperatureAnnualSinK, currentEastMeanMS, currentNorthMeanMS, currentEastAnnualCosMS, currentEastAnnualSinMS, currentNorthAnnualCosMS, currentNorthAnnualSinMS, currentSpeedMeanMS, oceanHeatTransportIndex,'''
if old not in text:
    raise SystemExit('worker seasonal result marker not found')
text = text.replace(old, new, 1)
old = '''result.seaSurfaceTemperatureMeanK.buffer, result.currentEastMeanMS.buffer, result.currentNorthMeanMS.buffer, result.currentSpeedMeanMS.buffer'''
new = '''result.seaSurfaceTemperatureMeanK.buffer, result.seaSurfaceTemperatureAnnualCosK.buffer, result.seaSurfaceTemperatureAnnualSinK.buffer, result.currentEastMeanMS.buffer, result.currentNorthMeanMS.buffer, result.currentEastAnnualCosMS.buffer, result.currentEastAnnualSinMS.buffer, result.currentNorthAnnualCosMS.buffer, result.currentNorthAnnualSinMS.buffer, result.currentSpeedMeanMS.buffer'''
if old not in text:
    raise SystemExit('worker seasonal transfer marker not found')
path.write_text(text.replace(old, new, 1))
