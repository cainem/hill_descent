use std::cmp::Ordering;


/// Sorts dimensions by diversity and returns the index of the most diverse one.
///
/// Diversity is determined first by the number of unique values (descending),
/// and then by standard deviation (descending) as a tie-breaker.
///
/// Returns `None` if the most diverse dimension has only one unique value,
/// indicating no meaningful diversity.
pub fn find_most_diverse_index(dimension_stats: Vec<(usize, f64)>) -> Option<usize> {
    let mut indexed_stats: Vec<_> = dimension_stats.into_iter().enumerate().collect();

    indexed_stats.sort_by(|(_, (a_unique, a_std)), (_, (b_unique, b_std))| {
        match b_unique.cmp(a_unique) {
            Ordering::Equal => b_std.partial_cmp(a_std).unwrap_or(Ordering::Equal),
            other => other,
        }
    });

    indexed_stats.first().and_then(|(index, (unique_count, _))| {
        if *unique_count > 1 {
            Some(*index)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_clear_winner_when_find_most_diverse_index_then_returns_correct_index() {
        // dim 1 has the most unique values
        let stats = vec![(2, 10.0), (5, 1.0), (3, 5.0)];
        assert_eq!(find_most_diverse_index(stats), Some(1));
    }

    #[test]
    fn given_tie_in_uniqueness_when_find_most_diverse_index_then_std_dev_breaks_tie() {
        // dim 0 and 1 have 5 unique values, but dim 1 has higher std dev
        let stats = vec![(5, 1.0), (5, 10.0), (3, 20.0)];
        assert_eq!(find_most_diverse_index(stats), Some(1));
    }

    #[test]
    fn given_no_diversity_when_find_most_diverse_index_then_returns_none() {
        // No dimension has more than 1 unique value
        let stats = vec![(1, 0.0), (1, 0.0), (1, 0.0)];
        assert_eq!(find_most_diverse_index(stats), None);
    }

    #[test]
    fn given_empty_stats_when_find_most_diverse_index_then_returns_none() {
        let stats: Vec<(usize, f64)> = vec![];
        assert_eq!(find_most_diverse_index(stats), None);
    }

    #[test]
    fn given_winner_has_one_unique_value_when_find_most_diverse_index_then_returns_none() {
        // Dim 1 would be the winner by std dev, but it only has 1 unique value
        let stats = vec![(1, 1.0), (1, 10.0), (1, 5.0)];
        assert_eq!(find_most_diverse_index(stats), None);
    }
}
