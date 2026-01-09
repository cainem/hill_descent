use xxhash_rust::xxh3::xxh3_64;

use crate::parameters::parameter::Parameter;

/// Direction of how adjustment is applied: add or subtract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionOfTravel {
    Add,
    Subtract,
}

/// A potential modification to a locus value.
#[derive(Debug, Clone, PartialEq)]
// Represents a potential modification to a locus value, including magnitude, direction, doubling/halving behaviour, and checksum.
pub struct LocusAdjustment {
    adjustment_value: Parameter,
    direction_of_travel: DirectionOfTravel,
    doubling_or_halving_flag: bool,
    checksum: u64,
}

impl LocusAdjustment {
    pub const ADJUSTMENT_VALUE_BOUND_PERCENTAGE: f64 = 0.1;

    /// Compute checksum from adjustment parameters.
    fn compute_checksum(
        adjustment_value: &Parameter,
        direction_of_travel: DirectionOfTravel,
        doubling_or_halving_flag: bool,
    ) -> u64 {
        let val_bytes = adjustment_value.get().to_le_bytes();
        let dir_byte = match direction_of_travel {
            DirectionOfTravel::Add => 0u8,
            DirectionOfTravel::Subtract => 1u8,
        };
        let flag_byte = if doubling_or_halving_flag { 1u8 } else { 0u8 };

        let data = [
            val_bytes[0],
            val_bytes[1],
            val_bytes[2],
            val_bytes[3],
            val_bytes[4],
            val_bytes[5],
            val_bytes[6],
            val_bytes[7],
            dir_byte,
            flag_byte,
        ];
        xxh3_64(&data)
    }

    /// Constructs a new LocusAdjustment, enforcing adjustment_value >= 0
    /// and computing the checksum via XXH3 over (value, direction, flag).
    pub fn new(
        adjustment_value: Parameter,
        direction_of_travel: DirectionOfTravel,
        doubling_or_halving_flag: bool,
    ) -> Self {
        assert!(
            adjustment_value.get() >= 0.0,
            "adjustment_value must be >= 0"
        );
        let checksum = Self::compute_checksum(
            &adjustment_value,
            direction_of_travel,
            doubling_or_halving_flag,
        );
        Self {
            adjustment_value,
            direction_of_travel,
            doubling_or_halving_flag,
            checksum,
        }
    }

    /// Returns the adjustment value.
    pub fn adjustment_value(&self) -> &Parameter {
        &self.adjustment_value
    }

    /// Returns the direction of travel for this adjustment.
    pub fn direction_of_travel(&self) -> DirectionOfTravel {
        self.direction_of_travel
    }

    /// Returns whether the adjustment value will double (true) or halve (false) on mutation.
    pub fn doubling_or_halving_flag(&self) -> bool {
        self.doubling_or_halving_flag
    }

    /// Returns the precomputed checksum over the adjustment state.
    pub fn checksum(&self) -> u64 {
        self.checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::parameter::Parameter; // Existing import // Existing import

    // E0 is available from file scope (super::E0 or just E0 if in same module level)
    // DirectionOfTravel is available from file scope (super::DirectionOfTravel or just DirectionOfTravel)

    // Existing tests
    #[test]
    fn given_valid_params_when_new_then_fields_set() {
        let p = Parameter::new(1.23);
        let adj1 = LocusAdjustment::new(p, DirectionOfTravel::Add, false);
        assert_eq!(adj1.adjustment_value(), &p);
        assert_eq!(adj1.direction_of_travel(), DirectionOfTravel::Add);
        assert!(!adj1.doubling_or_halving_flag());
        // deterministic checksum
        let adj2 = LocusAdjustment::new(p, DirectionOfTravel::Add, false);
        assert_eq!(adj1.checksum(), adj2.checksum());
    }

    #[test]
    #[should_panic]
    fn given_negative_adjustment_when_new_then_panic() {
        let pneg = Parameter::new(-1.0);
        let _ = LocusAdjustment::new(pneg, DirectionOfTravel::Subtract, true);
    }
}
