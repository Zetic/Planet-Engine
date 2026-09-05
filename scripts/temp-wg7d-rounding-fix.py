from pathlib import Path

path = Path('rust/interlink-worldgen/src/infill.rs')
text = path.read_text()
old = '''        );
        let mut changed = false;
        for &sample in &members {
            let old = evolution.evolved_solid_elevation_m[sample];
            let old_f64 = f64::from(old);
            if old_f64 >= fill_level {
                continue;
            }
            let new = fill_level as f32;
            if new <= old {
                continue;
            }
            post_infill_solid_elevation_m[sample] = new;
            lake_fill_depth_m[sample] = new - old;
            changed = true;
        }
'''
new = '''        );
        // The public terrain is f32. Quantize the solved depositional level downward so the
        // represented surface can never materialize more sediment volume than was delivered.
        let mut quantized_fill_level = fill_level as f32;
        if f64::from(quantized_fill_level) > fill_level {
            quantized_fill_level = quantized_fill_level.next_down();
        }
        let mut changed = false;
        for &sample in &members {
            let old = evolution.evolved_solid_elevation_m[sample];
            if old >= quantized_fill_level {
                continue;
            }
            let new = quantized_fill_level;
            post_infill_solid_elevation_m[sample] = new;
            lake_fill_depth_m[sample] = new - old;
            changed = true;
        }
'''
if old not in text:
    raise SystemExit('rounding anchor not found')
path.write_text(text.replace(old, new, 1))
