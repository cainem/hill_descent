//! Age increment and death checking implementation for organisms.

use super::IncrementAgeResult;

/// Increments the organism's age and checks if it should die.
///
/// # Arguments
///
/// * `current_age` - The organism's current age
/// * `max_age` - Maximum allowed age before death (as f64 for comparison with system parameters)
///
/// # Returns
///
/// Tuple of (IncrementAgeResult, new_age, is_dead).
///
/// # Algorithm
///
/// 1. Increment age by 1
/// 2. Check if new age exceeds max_age
/// 3. Return result with should_remove flag and new age
pub fn increment_age(current_age: usize, max_age: f64) -> (IncrementAgeResult, usize, bool) {
    let new_age = current_age + 1;
    let is_dead = (new_age as f64) > max_age;

    (
        IncrementAgeResult {
            should_remove: is_dead,
            new_age,
        },
        new_age,
        is_dead,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_young_organism_when_increment_age_then_age_increases() {
        let (result, new_age, _) = increment_age(0, 10.0);
        assert_eq!(new_age, 1);
        assert_eq!(result.new_age, 1);
    }

    #[test]
    fn given_young_organism_when_increment_age_then_should_remove_false() {
        let (result, _, is_dead) = increment_age(0, 10.0);
        assert!(!result.should_remove);
        assert!(!is_dead);
    }

    #[test]
    fn given_organism_at_max_age_when_increment_age_then_should_remove_true() {
        // max_age is 10.0, current age is 10, new age will be 11 which exceeds 10.0
        let (result, new_age, is_dead) = increment_age(10, 10.0);
        assert_eq!(new_age, 11);
        assert!(result.should_remove);
        assert!(is_dead);
    }

    #[test]
    fn given_organism_one_below_max_when_increment_age_then_should_remove_false() {
        // max_age is 10.0, current age is 9, new age will be 10 which equals max_age
        let (result, new_age, is_dead) = increment_age(9, 10.0);
        assert_eq!(new_age, 10);
        assert!(!result.should_remove);
        assert!(!is_dead);
    }

    #[test]
    fn given_organism_past_max_age_when_increment_age_then_is_dead_true() {
        // Already past max_age - still increments and reports dead
        let (result, new_age, is_dead) = increment_age(15, 10.0);
        assert_eq!(new_age, 16);
        assert!(result.should_remove);
        assert!(is_dead);
    }

    #[test]
    fn given_max_age_zero_when_increment_age_then_immediately_dead() {
        // Any organism immediately dies with max_age of 0
        let (result, new_age, is_dead) = increment_age(0, 0.0);
        assert_eq!(new_age, 1);
        assert!(result.should_remove);
        assert!(is_dead);
    }

    #[test]
    fn given_fractional_max_age_when_increment_age_then_compares_correctly() {
        // max_age = 5.5, so ages 1-5 are fine, age 6 exceeds it
        let (result1, _, is_dead1) = increment_age(4, 5.5);
        assert!(!result1.should_remove);
        assert!(!is_dead1);

        let (result2, _, is_dead2) = increment_age(5, 5.5);
        assert!(result2.should_remove);
        assert!(is_dead2);
    }
}
