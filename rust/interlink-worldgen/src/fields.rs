use crate::{random::hash_u16_field, FieldStatistics, WorldgenError};

pub const MAX_SYNTHETIC_SAMPLES: usize = 4_194_304;

#[derive(Clone, Debug, PartialEq)]
pub struct DenseU16Field {
    width: u32,
    height: u32,
    values: Vec<u16>,
}

impl DenseU16Field {
    pub fn from_values(width: u32, height: u32, values: Vec<u16>) -> Result<Self, WorldgenError> {
        let expected = checked_sample_count(width, height)?;
        if values.len() != expected {
            return Err(WorldgenError::InvalidDimensions(
                "field value count does not match width × height",
            ));
        }
        Ok(Self {
            width,
            height,
            values,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn values(&self) -> &[u16] {
        &self.values
    }

    pub fn statistics(&self) -> FieldStatistics {
        let mut minimum = u16::MAX;
        let mut maximum = u16::MIN;
        let mut sum = 0_u128;
        for value in &self.values {
            minimum = minimum.min(*value);
            maximum = maximum.max(*value);
            sum += u128::from(*value);
        }
        let sample_count = self.values.len() as u64;
        let mean = if sample_count == 0 {
            0.0
        } else {
            sum as f64 / sample_count as f64
        };
        FieldStatistics {
            sample_count,
            minimum,
            maximum,
            mean,
            hash: hash_u16_field(self.width, self.height, &self.values),
        }
    }
}

pub fn checked_sample_count(width: u32, height: u32) -> Result<usize, WorldgenError> {
    if width == 0 || height == 0 {
        return Err(WorldgenError::InvalidDimensions(
            "field dimensions must be non-zero",
        ));
    }
    let count =
        (width as usize)
            .checked_mul(height as usize)
            .ok_or(WorldgenError::InvalidDimensions(
                "field dimensions overflow addressable memory",
            ))?;
    if count > MAX_SYNTHETIC_SAMPLES {
        return Err(WorldgenError::InvalidDimensions(
            "WG-0 synthetic diagnostic is limited to 4,194,304 samples",
        ));
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_field_validates_shape() {
        assert!(DenseU16Field::from_values(2, 2, vec![1, 2, 3, 4]).is_ok());
        assert!(DenseU16Field::from_values(2, 2, vec![1, 2]).is_err());
        assert!(checked_sample_count(0, 1).is_err());
    }
}
