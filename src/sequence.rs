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
    T: QuadraticResidue
{
    /// The config/builder holds the parameters that define the sequence.
    pub config: RandomSequenceBuilder<T>,

    /// Internal iterator-only state.
    pub(crate) start_index: T,
    pub(crate) current_index: T,
    pub(crate) intermediate_offset: T,

    /// The end marker, required for the ExactSizeIterator so that we terminate correctly.
    pub(crate) ended: bool,
}

impl<T> RandomSequence<T>
where
    T: QuadraticResidue
{
    /// Get the next element in the sequence.
    #[inline]
    pub fn next(&mut self) -> Option<T> {
        let next = self.n_internal(self.start_index.wrapping_add(&self.current_index));
        self.current_index = match self.current_index.checked_add(&T::one()) {
            Some(v) => {
                self.ended = false;
                v
            },
            None => {
                if !self.ended {
                    self.ended = true;
                    self.current_index
                } else {
                    return None
                }
            },
        };
        Some(next)
    }

    /// Get the next element in the sequence, cycling the sequence once we reach the end.
    ///
    /// This will ignore the internal [RandomSequence::ended] marker, and potentially confuse an
    /// exact size iterator if it had reached the end.
    #[inline]
    pub fn wrapping_next(&mut self) -> T {
        let next = self.n_internal(self.start_index.wrapping_add(&self.current_index));
        self.current_index = self.current_index.wrapping_add(&T::one());
        next
    }

    /// Get the previous element in the sequence.
    #[inline]
    pub fn prev(&mut self) -> Option<T> {
        // decrement then compute, opposite to next()
        self.current_index = match self.current_index.checked_sub(&T::one()) {
            Some(v) => v,
            None => return None,
        };
        self.ended = false;
        Some(self.n_internal(self.start_index.wrapping_add(&self.current_index)))
    }

    /// Get the previous element in the sequence, cycling the sequence once we reach the start.
    #[inline]
    pub fn wrapping_prev(&mut self) -> T {
        // decrement then compute, opposite to next()
        self.current_index = self.current_index.wrapping_sub(&T::one());
        self.n_internal(self.start_index.wrapping_add(&self.current_index))
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
        let inner_residue = self.config.permute_qpr(index).wrapping_add(&self.intermediate_offset);
        self.config.permute_qpr(inner_residue ^ self.config.intermediate_xor)
    }

    /// Get the current position in the sequence.
    #[inline]
    pub fn index(&self) -> T {
        self.current_index
    }
}

macro_rules! impl_unsized_iterator {
    ($T:ident) => {
        impl Iterator for RandomSequence<$T> {
            type Item = $T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.next()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                ($T::MAX as usize, None)
            }
        }
    };
}

macro_rules! impl_exact_size_iterator {
    ($T:ident) => {
        impl Iterator for RandomSequence<$T> {
            type Item = $T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.next()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                ($T::MAX as usize + 1, Some($T::MAX as usize + 1))
            }
        }

        impl ExactSizeIterator for RandomSequence<$T> {}
    };
}

// Can only fit exact size iterators in types smaller than usize. As usize will have usize+1 elements.
impl_exact_size_iterator!(u8);
impl_exact_size_iterator!(u16);
#[cfg(target_pointer_width = "64")]
impl_exact_size_iterator!(u32);
#[cfg(target_pointer_width = "32")]
impl_unsized_iterator!(u32);
impl_unsized_iterator!(u64);
impl_unsized_iterator!(usize);

impl<T> DoubleEndedIterator for RandomSequence<T>
where
    T: QuadraticResidue,
    RandomSequence<T>: Iterator<Item = T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.prev()
    }
}

impl<T> From<RandomSequenceBuilder<T>> for RandomSequence<T>
where
    T: QuadraticResidue,
    RandomSequence<T>: Iterator<Item = T>,
{
    fn from(value: RandomSequenceBuilder<T>) -> Self {
        value.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::vec::Vec;

    use rand::rngs::OsRng;
    use statrs::distribution::{ChiSquared, ContinuousCDF};

    use super::*;

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    macro_rules! test_sequence {
        ($name:ident, $type:ident, $check:literal) => {
            #[test]
            fn $name() {
                let config = RandomSequenceBuilder::<$type>::new(0, 0);
                let sequence = config.into_iter();

                for (i, num) in std::iter::zip(0..10, sequence.clone()) {
                    assert_eq!(sequence.n(i as $type), num);
                }

                for (i, num) in std::iter::zip(0..10, sequence.clone().rev()) {
                    assert_eq!(sequence.n($type::MAX.wrapping_sub(i as $type)), num);
                }

                // check the exact size iterator ends correctly for u8 and u16
                if ($type::MAX as usize) < $check {
                    let nums_vec: Vec<$type> = config.into_iter().take($check + 10).collect();
                    assert_eq!(nums_vec.len(), $type::MAX as usize + 1);
                }

                // check that we see each value only once
                let nums: HashSet<$type> = config.into_iter().take($check).collect();
                assert_eq!(nums.len(), $check);

                // check sequence is send and sync (although index won't be synced between threads)
                is_send::<RandomSequence<$type>>();
                is_sync::<RandomSequence<$type>>();
            }
        };
    }

    test_sequence!(test_u8_sequence, u8, 256);
    test_sequence!(test_u16_sequence, u16, 65536);
    test_sequence!(test_u32_sequence, u32, 100_000);
    test_sequence!(test_u64_sequence, u64, 100_000);
    test_sequence!(test_usize_sequence, usize, 100_000);

    macro_rules! test_exact_size_iterator {
        ($name:ident, $type:ident) => {
            #[test]
            fn $name() {
                let config = RandomSequenceBuilder::<$type>::new(0, 0);
                let sequence = config.into_iter();
                assert_eq!(sequence.len(), $type::MAX as usize + 1);
            }
        };
    }

    test_exact_size_iterator!(test_u8_exact_size_iterator, u8);
    test_exact_size_iterator!(test_u16_exact_size_iterator, u16);
    #[cfg(target_pointer_width = "64")]
    test_exact_size_iterator!(test_u32_exact_size_iterator, u32);

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
