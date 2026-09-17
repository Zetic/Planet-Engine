from pathlib import Path

p = Path('src/worldgen/protocol.ts')
s = p.read_text()
# The initial wiring pattern can hit the earlier topography metric shape. Keep lithology identity
# on the cumulative result instead, where the stage actually exists.
s = s.replace('  lithosphereHash: string;\n  lithologyHash: string;\n  inheritanceHash: string;', '  lithosphereHash: string;\n  inheritanceHash: string;', 1)
anchor = '  lithologyStage: WorldgenStageMetadata;\n  bedrockClass: Uint8Array;'
if anchor not in s:
    raise SystemExit('protocol lithology stage anchor not found')
s = s.replace(anchor, '  lithologyStage: WorldgenStageMetadata;\n  lithologyHash: string;\n  bedrockClass: Uint8Array;', 1)
p.write_text(s)

p = Path('src/worldgen/worldgenWorker.ts')
s = p.read_text()
s = s.replace(', lithologyHash: output.lithology_hash_hex(), inheritanceHash:', ', inheritanceHash:', 1)
anchor = 'lithologyStage: { id: output.lithology_stage_id(), version: output.lithology_stage_version(), stageSeed: output.lithology_stage_seed_hex(), durationMs: 0 }, bedrockClass,'
if anchor not in s:
    raise SystemExit('worker lithology result anchor not found')
s = s.replace(anchor, 'lithologyStage: { id: output.lithology_stage_id(), version: output.lithology_stage_version(), stageSeed: output.lithology_stage_seed_hex(), durationMs: 0 }, lithologyHash: output.lithology_hash_hex(), bedrockClass,', 1)
p.write_text(s)
