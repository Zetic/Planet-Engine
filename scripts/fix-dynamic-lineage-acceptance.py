#!/usr/bin/env python3
from pathlib import Path

path = Path('rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs')
text = path.read_text()

old_decl = '''    let mut split_events = 0_usize;
    let mut split_event_children = BTreeSet::<u16>::new();
'''
new_decl = '''    let mut split_events = 0_usize;
    let mut split_event_children = BTreeSet::<u16>::new();
    let mut capture_event_children = BTreeSet::<u16>::new();
    let mut capture_partition_parents = BTreeSet::<u16>::new();
'''
if new_decl not in text:
    if old_decl not in text:
        raise SystemExit('missing lineage event declarations')
    text = text.replace(old_decl, new_decl, 1)

old_event = '''        if event.kind == HistoricalEventKind::Rift && event.fragment_a != event.fragment_b {
            split_events += 1;
            split_event_children.insert(event.fragment_a);
            split_event_children.insert(event.fragment_b);
        }
'''
new_event = '''        if event.kind == HistoricalEventKind::Rift && event.fragment_a != event.fragment_b {
            split_events += 1;
            split_event_children.insert(event.fragment_a);
            split_event_children.insert(event.fragment_b);
        }
        if event.kind == HistoricalEventKind::Capture && event.fragment_a != event.fragment_b {
            capture_event_children.insert(event.fragment_b);
            if let Some(parent) = history.fragments[event.fragment_b as usize].parent_fragment_id {
                capture_partition_parents.insert(parent);
            }
        }
'''
if new_event not in text:
    if old_event not in text:
        raise SystemExit('missing lineage event collection block')
    text = text.replace(old_event, new_event, 1)

old_check = '''    if parented_fragment_ids
        .iter()
        .any(|fragment| !split_event_children.contains(fragment))
    {
        return Err(format!(
            "{seed}: parented fragment lineage is missing explicit split-event provenance"
        ));
    }
'''
new_check = '''    if parented_fragment_ids.iter().any(|fragment| {
        if split_event_children.contains(fragment) || capture_event_children.contains(fragment) {
            return false;
        }
        history.fragments[*fragment as usize]
            .parent_fragment_id
            .map(|parent| !capture_partition_parents.contains(&parent))
            .unwrap_or(true)
    }) {
        return Err(format!(
            "{seed}: parented fragment lineage is missing explicit rift/capture partition provenance"
        ));
    }
'''
if new_check not in text:
    if old_check not in text:
        raise SystemExit('missing parented lineage provenance check')
    text = text.replace(old_check, new_check, 1)

path.write_text(text)
print('dynamic lineage acceptance updated')
