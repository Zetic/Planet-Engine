from pathlib import Path

for filename in ['index.html', 'worldgen-lab.html']:
    path = Path(filename)
    text = path.read_text()
    replacements = [
        ('<title>Planet Engine · WG-4 Lab</title>', '<title>Planet Engine · WG-5 Lab</title>'),
        ('PLANET ENGINE · THROUGH WG-4', 'PLANET ENGINE · THROUGH WG-5'),
        ('Generate one deterministic physical planet, then inspect its topology, inherited tectonic and geological state, lithosphere, and WG-4 pre-erosional topography from the same result.', 'Generate one deterministic physical planet through WG-5, then inspect topology, tectonics, geology, lithosphere, topography, seasonal climate, winds, surface currents, and moisture from the same result.'),
        ('value="interlink-wg4"', 'value="interlink-wg5"'),
        ('<strong>Current physical frontier: WG-4</strong>', '<strong>Current physical frontier: WG-5</strong>'),
        ('<script type="module" src="dist/worldgen/diagnostics/worldgenTopographyLabStandalone.js"></script>', '<script type="module" src="dist/worldgen/diagnostics/worldgenClimateLabStandalone.js"></script>'),
    ]
    for old, new in replacements:
        if old not in text:
            raise SystemExit(f'{filename}: marker not found: {old[:60]}')
        text = text.replace(old, new, 1)

    projection = '''      <label>Projection
        <select id="worldgen-projection">
          <option value="globe">Orthographic globe</option>
          <option value="map">Equirectangular map</option>
        </select>
      </label>
'''
    season = projection + '''      <label>Season / orbital phase
        <input id="worldgen-season" type="range" min="0" max="1000" value="0" step="1">
        <span id="worldgen-season-value">0.0% orbit</span>
      </label>
'''
    if projection not in text:
        raise SystemExit(f'{filename}: projection marker not found')
    text = text.replace(projection, season, 1)

    old_surface = '''          <optgroup label="Physical surface (WG-4)">
            <option value="relative-elevation" selected>Elevation above sea level</option>
            <option value="solid-elevation">Solid elevation / datum</option>
            <option value="land-water">Land / water</option>
            <option value="water-depth">Bathymetry / water depth</option>
          </optgroup>
'''
    new_surface = '''          <optgroup label="Climate · Radiation / temperature (WG-5)">
            <option value="temperature">Annual mean temperature</option>
            <option value="seasonal-temperature">Seasonal temperature</option>
            <option value="temperature-range">Annual temperature range</option>
            <option value="annual-insolation">Annual mean insolation</option>
            <option value="seasonal-insolation">Seasonal insolation amplitude</option>
            <option value="sst">Annual mean sea-surface temperature</option>
            <option value="seasonal-sst">Seasonal sea-surface temperature</option>
          </optgroup>
          <optgroup label="Climate · Atmosphere / ocean (WG-5)">
            <option value="winds">Seasonal prevailing winds</option>
            <option value="wind-speed">Annual mean wind speed</option>
            <option value="surface-pressure">Surface atmospheric pressure</option>
            <option value="currents">Seasonal surface ocean currents</option>
            <option value="current-speed">Annual mean current speed</option>
            <option value="ocean-heat">Ocean heat transport</option>
          </optgroup>
          <optgroup label="Climate · Moisture / cryosphere (WG-5)">
            <option value="humidity">Atmospheric specific humidity</option>
            <option value="precipitation">Annual precipitation</option>
            <option value="precip-seasonality">Precipitation seasonality</option>
            <option value="potential-evaporation">Potential evaporation</option>
            <option value="moisture-balance">Moisture balance</option>
            <option value="aridity">Aridity index</option>
            <option value="snowfall">Snowfall fraction</option>
            <option value="persistent-snow">Persistent snow potential</option>
            <option value="sea-ice">Sea-ice potential</option>
          </optgroup>
          <optgroup label="Physical surface (WG-4)">
            <option value="physical-elevation" selected>Physical elevation / bathymetry</option>
            <option value="relative-elevation">Elevation above sea level</option>
            <option value="solid-elevation">Solid elevation / datum</option>
            <option value="land-water">Land / water</option>
            <option value="water-depth">Bathymetry / water depth</option>
          </optgroup>
'''
    if old_surface not in text:
        raise SystemExit(f'{filename}: surface options marker not found')
    text = text.replace(old_surface, new_surface, 1)

    old_note = '''          <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, and initial-topography pipeline. Diagnostic modes inspect that same generated planet rather than regenerating earlier stages.</p>
          <p>WG-4 adds crustal isostatic support, oceanic age subsidence, collision/ridge/rift/subduction morphology, inherited basin tendency, broad mantle support, lithospheric mechanical filtering, and a water-volume-derived sea-level solution.</p>
          <p>This remains pre-erosional physical topography. Climate, drainage, river incision, sediment transport, glaciation, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>
'''
    new_note = '''          <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, and WG-5 coupled-climate pipeline. Every diagnostic mode inspects that same generated planet.</p>
          <p>WG-5 adds seasonal orbital forcing, land/ocean thermal response, prevailing winds, wind-driven surface ocean currents, SST heat transport, atmospheric moisture, orographic precipitation, aridity, and snow/sea-ice potential. The season slider reconstructs stored climatology; it does not rerun the solver.</p>
          <p>The physical surface remains pre-erosional. Drainage, river incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>
'''
    if old_note not in text:
        raise SystemExit(f'{filename}: note marker not found')
    text = text.replace(old_note, new_note, 1)
    path.write_text(text)
