use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingSub};

use crate::sequence::RandomSequence;

/// The configuration for [RandomSequence], a random unique sequence generator.
///
/// These variables define the entire sequence and should not be modified with the exception of
/// `seed_base` and `seed_offset` during initialisation.
///
/// The builder defines the internal properties of the sequence, and serialization includes all
/// of the properties to preserve the sequence between crate versions which may change the fixed
/// values between minor versions.
///
/// Crate versioning will bump:
/// - _Minor version_: when the hard coded parameters are updated in favour of better ones. It is
///    safe to serialize the [RandomSequenceBuilder] between minor versions.
/// - _Major version_: when the sequence generation logic fundamentally changes the sequence,
///    meaning it would be potentially unsafe to serialize the [RandomSequenceBuilder] between
///    major crate version changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomSequenceBuilder<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    /// The sequence seed value.
    pub seed: T,

    /// Should be the largest prime number that fits in type `T` and satisfied `prime = 3 mod 4`.
    pub prime: T,

    /// The maximum value in the sequence.
    pub max: T,

    /// A value that provides some constant noise in the sequence. Chosen to be a fairly large prime
    /// number in type T to avoid cycles.
    pub intermediate_a: T,

    /// A value that provides some variable noise in the sequence. Determined by the seed.
    pub intermediate_b: T,
}

impl<T> RandomSequenceBuilder<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    /// Set the maximum value for the sequence. Will run a check to discover the largest suitable
    /// prime number. Prime number discovery is fairly quick, but it is recommended not to run this
    /// in a very tight loop.
    ///
    /// Note: `max` is _inclusive_, so the sequence will include `max`.
    pub fn with_max(self, max: T) -> Self {
        let prime = Self::find_suitable_prime(max);
        Self { max, prime, ..self }
    }

    /// Discover the largest suitable prime number that satisfies:
    /// - prime < max
    /// - prime = 3 mod 4 (`prime % 4 == 3`)
    pub fn find_suitable_prime(max: T) -> T {
        let mut number = max;

        // 0, 1, 2 are all suitable primes for very small sequences
        if number <= T::one() + T::one() + T::one() {
            return number;
        }

        // fast check for even numbers, operation equivalent to p % 2 == 0
        if (number & T::one()) == T::zero() {
            number = number - T::one();
        }

        // suggest a suitable prime number
        while number > T::one() + T::one() + T::one() {
            // fast check for p = 3 mod 4, operation equivalent to p % 4 == 3
            if number & (T::one() + T::one() + T::one()) == (T::one() + T::one() + T::one()) {
                // 3 mod 4 is fast, so only search for a prime if that succeeds
                if machine_prime::is_prime(number.as_()) {
                    break;
                }
            }

            // decrement by two to skip even numbers
            number = number - T::one() - T::one();
        }

        number
    }

    /// Intermediary function to compute the quadratic prime residue.
    #[inline]
    pub(crate) fn permute_qpr(&self, x: T) -> T {
        // The small set of integers out of range are mapped to themselves.
        if x >= self.prime && self.prime > T::one() {
            // for small sequences this adds noise
            if self.intermediate_b & T::one() == T::zero() {
                if x == self.prime {
                    return self.max
                } else if x == self.max {
                    return self.prime;
                }
            }

            return x;
        }

        // Extra noise for small sequences to flip 0/1 depending on seed, otherwise 0->0 and 1->1
        if x <= T::one() {
            // the & clause is to ensure self.max = 0 returns 0
            return (x ^ (self.seed & T::one())) & (if self.max > T::zero() { T::one() } else { T::zero() });
        }

        // (x * x) % prime; but done safely to avoid integer overflow on x * x
        let residue = x.residue(self.prime);

        // Op: `self.prime / 2` the bit shift is used to get around rust types
        if x <= self.prime >> 1 {
            residue
        } else {
            self.prime - residue
        }
    }
}

impl<T> IntoIterator for RandomSequenceBuilder<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + AsPrimitive<u64> + QuadraticResidue
{
    type Item = T;
    type IntoIter = RandomSequence<T>;

    /// Build a [RandomSequence] iterator from this config.
    fn into_iter(self) -> Self::IntoIter {
        let mut start_index = T::zero();
        if self.max > T::zero() {
            start_index = self.permute_qpr(self.permute_qpr(self.seed).wrapping_add(&self.intermediate_b));
        }


        RandomSequence {
            config: self,
            start_index,
            current_index: start_index,
        }
    }
}

/// Hardcoded constant used to add some noise to the seed and build the intermediate variables
/// from the seed.
const SEED_NOISE: u64 = 6624854654305503467;

macro_rules! impl_seed {
    ($type:ident, $prime:literal) => {
        impl RandomSequenceBuilder<$type> {
            /// Initialise this RandomSequenceBuilder with a particular seed.
            ///
            /// Note that how seeds are used is liable to change between crate minor version
            /// increments, and so if consistency is important, please correctly serialize the
            /// [RandomSequenceBuilder] struct rather than relying on the seed.
            pub fn seed(seed: u64) -> Self {
                Self {
                    // final bit determines 0/1 swapping
                    seed: ((seed ^ SEED_NOISE).wrapping_add(SEED_NOISE)) as $type,
                    // constant intermediate
                    intermediate_a: SEED_NOISE as $type,
                    // variable intermediate, we want seed to determine odd vs even addition
                    intermediate_b: (seed >> 1).wrapping_sub(SEED_NOISE + 1) as $type,
                    prime: $prime as $type,
                    max: $type::MAX,
                }
            }
        }
    };
}

impl_seed!(u8, 251);
impl_seed!(u16, 65519);
impl_seed!(u32, 4294967291);
impl_seed!(u64, 18446744073709551427);
#[cfg(target_pointer_width = "32")]
impl_seed!(usize, 4294967291u32);
#[cfg(target_pointer_width = "64")]
impl_seed!(usize, 18446744073709551427u64);
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("Unsupported pointer width, add new spec for usize here.");

pub trait QuadraticResidue {
    /// Compute the quadratic residue of this integer against a prime.
    fn residue(self, prime: Self) -> Self;

    /// Perform safe modular addition, avoiding overflow.
    fn modulo_add(self, b: Self, modulo: Self) -> Self;
}

macro_rules! impl_residue {
    ($base_type:ident, $larger_type:ident) => {
        impl QuadraticResidue for $base_type {
            /// Compute the quadratic residue of this number against a prime.
            fn residue(self, prime: Self) -> Self {
                ((self as $larger_type * self as $larger_type) % prime as $larger_type) as Self
            }

            /// Do modular addition with a larger type to avoid overflow.
            fn modulo_add(self, b: Self, max: Self) -> Self {
                if max == Self::MAX {
                    return self.wrapping_add(b);
                }
                ((self as $larger_type + b as $larger_type) % (max as $larger_type + 1)) as Self
            }
        }
    };
}

impl_residue!(u8, u16);
impl_residue!(u16, u32);
impl_residue!(u32, u64);
impl_residue!(u64, u128);
#[cfg(target_pointer_width = "32")]
impl_residue!(usize, u64);
#[cfg(target_pointer_width = "64")]
impl_residue!(usize, u128);
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("Unsupported pointer width, add new spec fo usize here.");

#[cfg(test)]
mod tests {
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;

    use super::*;

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    #[test]
    fn check_seed_constant_is_prime() {
        assert_eq!(SEED_NOISE, RandomSequenceBuilder::<u64>::find_suitable_prime(SEED_NOISE));
    }

    macro_rules! test_config {
        ($name:ident, $type:ident, $check:literal) => {
            #[test]
            fn $name() {
                let config = RandomSequenceBuilder::<$type>::seed(0);
                let config_orig = config.clone();

                // check the configured prime number satisfies the requirements
                // is_prime crate makes this very quick for u64
                let is_prime_res = machine_prime::is_prime(config.prime as u64);
                let is_3_mod_4_res = config.prime as u64 % 4 == 3;

                let mut found_number = config.prime;
                if !is_prime_res || !is_3_mod_4_res {
                    found_number = RandomSequenceBuilder::<$type>::find_suitable_prime(config.max);
                }

                assert!(is_prime_res, "{} is not prime, suggested prime: {}", config.prime, found_number);
                assert!(is_3_mod_4_res, "{} = 3 mod 4 doesn't hold, suggested prime: {}", config.prime, found_number);

                // check config can be cloned and equality tested
                let sequence = config.into_iter();
                assert_eq!(sequence.config, config_orig);

                // check permute_qpr for uniqueness
                const CHECK: usize = $check;
                let mut nums = HashMap::<$type, usize>::new();
                for i in 0..CHECK {
                    let num = config.permute_qpr(i as $type);
                    match nums.entry(num) {
                        Entry::Vacant(v) => {
                            v.insert(i);
                        }
                        Entry::Occupied(o) => {
                            panic!("Duplicate number {} at index {} and {}", num, o.get(), i);
                        }
                    }
                }
                assert_eq!(nums.len(), (0..CHECK).len());

                // check builder is send and sync
                is_send::<RandomSequenceBuilder<$type>>();
                is_sync::<RandomSequenceBuilder<$type>>();

                // test with_max
                let config = RandomSequenceBuilder::<$type>::seed(0).with_max(100 as $type);
                assert_eq!(config.max, 100);
                assert_eq!(config.prime, 83);
            }
        };
    }

    test_config!(test_u8_config, u8, 256);
    test_config!(test_u16_config, u16, 65536);
    test_config!(test_u32_config, u32, 100_000);
    test_config!(test_u64_config, u64, 100_000);
    test_config!(test_usize_config, usize, 100_000);

    #[test]
    fn test_find_suitable_prime() {
        assert_eq!(RandomSequenceBuilder::<u64>::find_suitable_prime(u64::MAX), RandomSequenceBuilder::<u64>::seed(0).prime);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(u8::MAX as u32), 251);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(101), 83);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(100), 83);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(7), 7);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(6), 3);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(5), 3);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(4), 3);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(3), 3);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(2), 2);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(1), 1);
        assert_eq!(RandomSequenceBuilder::<u32>::find_suitable_prime(0), 0);
    }
}
