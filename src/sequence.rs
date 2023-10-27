use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingSub};

use crate::builder::{QuadraticResidue, RandomSequenceBuilder};

/// Generate a deterministic pseudo-random sequence of unique numbers.
///
/// Not cryptographically secure.
///
/// Properties:
/// - The sequence is deterministic and repeatable.
/// - The sequence will only include each number once (every index is unique).
/// - Computing the value for any random index in the sequence is an O(1) operation.
///
/// Based on the article by @preshing:
/// Article: http://preshing.com/20121224/how-to-generate-a-sequence-of-unique-random-integers/
/// Source: https://github.com/preshing/RandomSequence/blob/master/randomsequence.h
#[derive(Debug, Clone)]
pub struct RandomSequence<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    /// The config/builder holds the parameters that define the sequence.
    pub config: RandomSequenceBuilder<T>,

    /// Internal iterator-only state.
    pub(crate) start_index: T,
    pub(crate) current_index: T,
}

impl<T> RandomSequence<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    /// Get the next element in the sequence.
    #[inline]
    pub fn next(&mut self) -> T {
        let next = self.n_internal(self.current_index);
        self.current_index = self.current_index.wrapping_add(&T::one());
        next
    }

    /// Get the previous element in the sequence.
    #[inline]
    pub fn prev(&mut self) -> T {
        // decrement then compute, opposite to next()
        self.current_index = self.current_index.wrapping_sub(&T::one());
        self.n_internal(self.current_index)
    }

    /// Get the nth element in the sequence.
    #[inline]
    pub fn n(&self, index: T) -> T {
        let actual_index = self.start_index.wrapping_add(&index);
        self.n_internal(actual_index)
    }

    /// Get the nth element in the sequence, but using the absolute index rather than relative to `start_index`.
    ///
    /// `qpr(qpr(index + intermediate_offset) ^ intermediate_xor)`
    #[inline(always)]
    fn n_internal(&self, index: T) -> T {
        let mut actual_index = index;
        if self.config.max != T::max_value() {
            actual_index = actual_index % (self.config.max + T::one());
        }

        let inner_residue = self.config.permute_qpr(actual_index.modulo_add(self.config.intermediate_b, self.config.max));
        self.config.permute_qpr(inner_residue.modulo_add(self.config.intermediate_a, self.config.max))
    }

    /// Get the current position in the sequence.
    #[inline]
    pub fn index(&self) -> T {
        self.current_index.wrapping_sub(&self.start_index)
    }
}

impl<T> Iterator for RandomSequence<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

impl<T> DoubleEndedIterator for RandomSequence<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.prev())
    }
}

impl<T> From<RandomSequenceBuilder<T>> for RandomSequence<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    fn from(value: RandomSequenceBuilder<T>) -> Self {
        value.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::vec;
    use std::vec::Vec;

    use rand::rngs::OsRng;
    use statrs::distribution::{ChiSquared, ContinuousCDF};

    use super::*;

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    macro_rules! assert_sequence {
        ($seed:literal, $cmp:expr) => {
            let cmp: Vec<usize> = $cmp;
            let config = RandomSequenceBuilder::<usize>::seed($seed).with_max(cmp.len() - 1);
            let sequence: Vec<usize> = config.into_iter().take(cmp.len()).collect();
            assert_eq!(sequence, cmp, "seed {} did not produce correct vec of len {}", $seed, cmp.len());
        };
    }

    /// Check the randomness of small sequences. Generated via `examples/assertions.rs`.
    #[test]
    fn test_small_max() {
        assert_sequence!(0, vec![1, 4, 5, 2, 3, 0]);
        assert_sequence!(1, vec![3, 0, 4, 5, 2, 1]);
        assert_sequence!(2, vec![1, 2, 3, 4, 5, 0]);
        assert_sequence!(3, vec![5, 0, 2, 3, 4, 1]);
        assert_sequence!(4, vec![5, 2, 3, 0, 1, 4]);
        assert_sequence!(5, vec![4, 5, 2, 1, 3, 0]);
        assert_sequence!(6, vec![3, 4, 5, 0, 1, 2]);
        assert_sequence!(7, vec![2, 3, 4, 1, 5, 0]);
        assert_sequence!(8, vec![1, 4, 5, 2, 3, 0]);
        assert_sequence!(9, vec![3, 0, 4, 5, 2, 1]);

        assert_sequence!(0, vec![0, 2, 4, 3, 1]);
        assert_sequence!(1, vec![0, 1, 4, 2, 3]);
        assert_sequence!(2, vec![1, 2, 3, 4, 0]);
        assert_sequence!(3, vec![1, 0, 3, 2, 4]);
        assert_sequence!(4, vec![3, 1, 0, 2, 4]);
        assert_sequence!(5, vec![2, 3, 0, 1, 4]);
        assert_sequence!(6, vec![4, 0, 1, 2, 3]);
        assert_sequence!(7, vec![2, 4, 1, 0, 3]);
        assert_sequence!(8, vec![0, 2, 4, 3, 1]);
        assert_sequence!(9, vec![0, 1, 4, 2, 3]);

        assert_sequence!(0, vec![1, 2, 3, 0]);
        assert_sequence!(1, vec![3, 0, 2, 1]);
        assert_sequence!(2, vec![1, 2, 3, 0]);
        assert_sequence!(3, vec![3, 0, 2, 1]);
        assert_sequence!(4, vec![1, 2, 3, 0]);
        assert_sequence!(5, vec![3, 0, 2, 1]);
        assert_sequence!(6, vec![1, 2, 3, 0]);
        assert_sequence!(7, vec![3, 0, 2, 1]);
        assert_sequence!(8, vec![1, 2, 3, 0]);
        assert_sequence!(9, vec![3, 0, 2, 1]);

        assert_sequence!(0, vec![1, 2, 0]);
        assert_sequence!(1, vec![2, 0, 1]);
        assert_sequence!(2, vec![1, 2, 0]);
        assert_sequence!(3, vec![2, 0, 1]);
        assert_sequence!(4, vec![0, 1, 2]);
        assert_sequence!(5, vec![1, 2, 0]);
        assert_sequence!(6, vec![0, 1, 2]);
        assert_sequence!(7, vec![1, 2, 0]);
        assert_sequence!(8, vec![1, 2, 0]);
        assert_sequence!(9, vec![2, 0, 1]);

        assert_sequence!(0, vec![0, 1]);
        assert_sequence!(1, vec![0, 1]);
        assert_sequence!(2, vec![1, 0]);
        assert_sequence!(3, vec![1, 0]);
        assert_sequence!(4, vec![0, 1]);
        assert_sequence!(5, vec![0, 1]);
        assert_sequence!(6, vec![1, 0]);
        assert_sequence!(7, vec![1, 0]);
        assert_sequence!(8, vec![0, 1]);
        assert_sequence!(9, vec![0, 1]);

        assert_sequence!(0, vec![0]);
        assert_sequence!(1, vec![0]);
        assert_sequence!(2, vec![0]);
        assert_sequence!(3, vec![0]);
        assert_sequence!(4, vec![0]);
        assert_sequence!(5, vec![0]);
        assert_sequence!(6, vec![0]);
        assert_sequence!(7, vec![0]);
        assert_sequence!(8, vec![0]);
        assert_sequence!(9, vec![0]);
    }

    macro_rules! test_sequence {
        ($name:ident, $type:ident, $check:literal, $max:literal) => {
            #[test]
            fn $name() {
                let config = RandomSequenceBuilder::<$type>::seed(0);
                let sequence = config.into_iter();

                for (i, num) in std::iter::zip(0..10, sequence.clone()) {
                    assert_eq!(sequence.n(i as $type), num);
                }

                for (i, num) in std::iter::zip(0..10, sequence.clone().rev()) {
                    assert_eq!(sequence.n($type::MAX.wrapping_sub(i as $type)), num);
                }

                let nums: HashSet<$type> = config.into_iter().take($check).collect();
                assert_eq!(nums.len(), $check);

                // check sequence is send and sync (although index won't be synced between threads)
                is_send::<RandomSequence<$type>>();
                is_sync::<RandomSequence<$type>>();

                // check sequence with max
                let values: Vec<$type> = config.with_max($max).into_iter().take($check).collect();
                assert!(*values.iter().max().unwrap() <= $max);
                let nums: HashSet<$type> = values.into_iter().collect();
                assert_eq!(nums.len(), $max + 1);
            }
        };
    }

    test_sequence!(test_u8_sequence, u8, 256, 100);
    test_sequence!(test_u16_sequence, u16, 65536, 1_000);
    test_sequence!(test_u32_sequence, u32, 100_000, 10_000);
    test_sequence!(test_u64_sequence, u64, 100_000, 10_000);

    macro_rules! test_distribution {
        ($name:ident, $type:ident, $check:literal) => {
            #[ignore]  // ChiSquared p value is too unreliable
            #[test]
            fn $name() {
                const BUCKETS: usize = 100;
                let config = RandomSequenceBuilder::<$type>::rand(&mut OsRng);

                // compute a normalised histogram over the sequence with BUCKETS buckets, where each bucket value
                // is the percentage of values that fall into this bucket
                let mut data_buckets: HashMap<usize, usize> = HashMap::with_capacity(BUCKETS + 1);
                config
                    .into_iter()
                    .take($check)
                    .map(|i| ((i as f64 / $type::MAX as f64) * BUCKETS as f64) as usize)
                    .for_each(|i| *data_buckets.entry(i).or_insert(0) += 1);
                let data_buckets: Vec<f64> = (0..=BUCKETS)
                    .map(|i| *data_buckets.get(&i).unwrap_or(&0) as f64)
                    .collect();

                // compute the probability of each bucket being hit, assuming a uniform distribution.
                // careful for u8 where we have 256 for only 100 buckets; and so some buckets have 2 vs 3 expected values,
                // as this represents the percentage of values that should fall into each bucket assuming perfectly uniform.
                let mut uniform_buckets: Vec<f64> = (0..BUCKETS)
                    .map(|_| ($check as f64 / BUCKETS as f64))
                    .collect();
                uniform_buckets.push($check as f64 / $type::MAX as f64); // last bucket for value=$type::MAX

                // compute chi-squared statistic
                assert_eq!(data_buckets.len(), uniform_buckets.len(), "Data and uniform buckets logic issue.");
                let chi_squared = std::iter::zip(data_buckets.iter(), uniform_buckets.iter())
                    .map(|(x, e)| (x - e).powi(2) / e)
                    .sum::<f64>();

                // compute p_value from chi-squared statistic
                let chi_dist = ChiSquared::new((BUCKETS - 1) as f64).unwrap();
                let p_value = 1.0 - chi_dist.cdf(chi_squared);

                // FIXME: choose a better test, because this doesn't strictly confirm the uniform distribution
                //   and there is a suspiciously large amount of variance in the p_values between test runs.
                // p_value <= 0.05 would say with 95% certainty that this distribution is _not_ uniform
                assert!(p_value > 0.05, "Unexpectedly rejected the null hypothesis with high probability. stat: {}, p: {}", chi_squared, p_value);
            }
        };
    }

    test_distribution!(test_u8_distribution, u8, 256);
    test_distribution!(test_u16_distribution, u16, 65536);
    test_distribution!(test_u32_distribution, u32, 100_000);
    test_distribution!(test_u64_distribution, u64, 100_000);
    test_distribution!(test_usize_distribution, usize, 100_000);
}
