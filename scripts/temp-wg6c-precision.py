from pathlib import Path

path = Path('rust/interlink-worldgen/src/lakes.rs')
text = path.read_text()
replacements = {
    '    let mut lake_fraction = vec![0.0_f32; count];': '    let mut lake_fraction_exact = vec![0.0_f64; count];',
    '                lake_fraction[sample] = fraction as f32;': '                lake_fraction_exact[sample] = fraction;',
    '            lake_fraction[sample] = 1.0;': '            lake_fraction_exact[sample] = 1.0;',
    '            let fraction = f64::from(lake_fraction[sample]);': '            let fraction = lake_fraction_exact[sample];',
    '                f64::from(local_runoff_m3_s[i]) * (1.0 - f64::from(lake_fraction[i]));': '                f64::from(local_runoff_m3_s[i]) * (1.0 - lake_fraction_exact[i]);',
    '        if lake_fraction[i] > 0.0 {': '        if lake_fraction_exact[i] > 0.0 {',
    '        if lake_fraction[i] <= 0.0 {': '        if lake_fraction_exact[i] <= 0.0 {',
    '                && lake_fraction[i] <= 0.0)': '                && lake_fraction_exact[i] <= 0.0)',
    '.map(|i| f64::from(local_runoff_m3_s[i]) * (1.0 - f64::from(lake_fraction[i])))': '.map(|i| f64::from(local_runoff_m3_s[i]) * (1.0 - lake_fraction_exact[i]))',
    '    Ok(LakeCore {\n        lake_id,\n        lake_kind,\n        lake_fraction,': "    let lake_fraction = lake_fraction_exact\n        .iter()\n        .map(|value| *value as f32)\n        .collect::<Vec<_>>();\n\n    Ok(LakeCore {\n        lake_id,\n        lake_kind,\n        lake_fraction,",
}
for old, new in replacements.items():
    if old not in text:
        raise SystemExit(f'marker not found: {old}')
    text = text.replace(old, new)
path.write_text(text)
