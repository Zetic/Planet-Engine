pub const FLOW_REGIME_NONE: u8 = 0;
pub const FLOW_REGIME_INTERMITTENT: u8 = 1;
pub const FLOW_REGIME_PERENNIAL: u8 = 2;
pub const SEASONAL_FLOW_PRESENCE_EPSILON_M3_S: f64 = 1.0e-6;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SeasonalFlowClassification {
    pub presence_fraction: Vec<f32>,
    pub regime: Vec<u8>,
    pub dry_sample_count: u32,
    pub intermittent_sample_count: u32,
    pub perennial_sample_count: u32,
}

pub(crate) fn classify_realized_flow(
    phase_realized_discharge_m3_s: &[f32],
    submerged_mask: &[u8],
    phase_count: usize,
) -> Result<SeasonalFlowClassification, &'static str> {
    if phase_count == 0 {
        return Err("seasonal flow classification requires at least one orbital phase");
    }
    let sample_count = submerged_mask.len();
    if phase_realized_discharge_m3_s.len() != sample_count * phase_count {
        return Err("seasonal flow classification fields must align with phase/topology dimensions");
    }

    let mut presence_fraction = vec![0.0_f32; sample_count];
    let mut regime = vec![FLOW_REGIME_NONE; sample_count];
    let mut dry_sample_count = 0_u32;
    let mut intermittent_sample_count = 0_u32;
    let mut perennial_sample_count = 0_u32;

    for sample in 0..sample_count {
        if submerged_mask[sample] != 0 {
            continue;
        }
        let mut wet_phases = 0_usize;
        for phase in 0..phase_count {
            let discharge = f64::from(
                phase_realized_discharge_m3_s[phase * sample_count + sample],
            );
            if !discharge.is_finite() || discharge < 0.0 {
                return Err("seasonal realized discharge must be finite and non-negative");
            }
            if discharge > SEASONAL_FLOW_PRESENCE_EPSILON_M3_S {
                wet_phases += 1;
            }
        }
        presence_fraction[sample] = wet_phases as f32 / phase_count as f32;
        regime[sample] = if wet_phases == 0 {
            dry_sample_count += 1;
            FLOW_REGIME_NONE
        } else if wet_phases == phase_count {
            perennial_sample_count += 1;
            FLOW_REGIME_PERENNIAL
        } else {
            intermittent_sample_count += 1;
            FLOW_REGIME_INTERMITTENT
        };
    }

    Ok(SeasonalFlowClassification {
        presence_fraction,
        regime,
        dry_sample_count,
        intermittent_sample_count,
        perennial_sample_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_dry_intermittent_and_perennial_land_samples() {
        let phases = 4;
        let submerged = [0_u8, 0, 0, 1];
        let phase_major = [
            0.0_f32, 2.0, 3.0, 9.0,
            0.0, 0.0, 4.0, 9.0,
            0.0, 1.0, 5.0, 9.0,
            0.0, 0.0, 6.0, 9.0,
        ];
        let classified = classify_realized_flow(&phase_major, &submerged, phases).unwrap();
        assert_eq!(
            classified.regime,
            [
                FLOW_REGIME_NONE,
                FLOW_REGIME_INTERMITTENT,
                FLOW_REGIME_PERENNIAL,
                FLOW_REGIME_NONE,
            ]
        );
        assert_eq!(classified.presence_fraction, [0.0, 0.5, 1.0, 0.0]);
        assert_eq!(classified.dry_sample_count, 1);
        assert_eq!(classified.intermittent_sample_count, 1);
        assert_eq!(classified.perennial_sample_count, 1);
    }

    #[test]
    fn numerical_traces_below_presence_epsilon_remain_dry() {
        let classified = classify_realized_flow(
            &[0.5e-6_f32, 2.0e-6, 0.0, 0.0],
            &[0_u8],
            4,
        )
        .unwrap();
        assert_eq!(classified.regime[0], FLOW_REGIME_INTERMITTENT);
        assert_eq!(classified.presence_fraction[0], 0.25);
    }
}
