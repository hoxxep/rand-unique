use num_traits::{PrimInt, WrappingAdd, WrappingSub};

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
    T: PrimInt + WrappingAdd + WrappingSub + QuadraticResidue
{
    pub seed_base: T,
    pub seed_offset: T,

    /// A value used as an xor during initialisation for `start_index = f(seed_base, init_base)` to
    /// deterministically pseudo-randomise it.
    pub init_base: T,

    /// A value used as an xor during initialisation for `offset = f(seed_offset, init_offset)` to
    /// deterministically pseudo-randomise it.
    pub init_offset: T,

    /// Should be the largest prime number that fits in type `T` and satisfied `prime = 3 mod 4`.
    pub prime: T,

    /// A value that provides some noise from the xor to generate a pseudo-uniform distribution.
    pub intermediate_xor: T,
}

impl<T> RandomSequenceBuilder<T>
where
    T: PrimInt + WrappingAdd + WrappingSub + QuadraticResidue
{
    /// Initialise a config from stored settings. Not recommended unless you know what you're doing,
    /// or these values have been taken from an already serialized RandomSequenceBuilder.
    ///
    /// Prefer [RandomSequenceBuilderInit::new] instead.
    pub unsafe fn from_spec(
        seed_base: T,
        seed_offset: T,
        init_base: T,
        init_offset: T,
        prime: T,
        intermediate_xor: T,
    ) -> Self {
        Self {
            seed_base,
            seed_offset,
            init_base,
            init_offset,
            prime,
            intermediate_xor,
        }
    }

    /// Intermediary function to compute the quadratic prime residue.
    #[inline]
    pub(crate) fn permute_qpr(&self, x: T) -> T {
        // The small set of integers out of range are mapped to themselves.
        if x >= self.prime {
            return x;
        }

        // (x * x) % prime; but done safely to avoid integer overflow on x * x
        // let xm = MontgomeryInt::new(x, &self.prime);
        // let residue = (xm * xm).residue();

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
    T: PrimInt + WrappingAdd + WrappingSub + QuadraticResidue
{
    type Item = T;
    type IntoIter = RandomSequence<T>;

    /// Build a [RandomSequence] iterator from this config.
    fn into_iter(self) -> Self::IntoIter {
        let start_index = self.permute_qpr(self.permute_qpr(self.seed_base).wrapping_add(&self.init_base));
        let intermediate_offset = self.permute_qpr(self.permute_qpr(self.seed_offset).wrapping_add(&self.init_offset));

        RandomSequence {
            config: self,
            start_index,
            current_index: start_index,
            intermediate_offset,
        }
    }
}

impl RandomSequenceBuilder<u8> {
    pub fn new(seed_base: u8, seed_offset: u8) -> Self {
        Self {
            seed_base,
            seed_offset,
            init_base: 167,
            init_offset: 181,
            prime: 251,
            intermediate_xor: 137,
        }
    }
}

impl RandomSequenceBuilder<u16> {
    pub fn new(seed_base: u16, seed_offset: u16) -> Self {
        Self {
            seed_base,
            seed_offset,
            init_base: 0x682f,
            init_offset: 0x4679,
            prime: 65519,
            intermediate_xor: 0x5bf0,
        }
    }
}

impl RandomSequenceBuilder<u32> {
    pub fn new(seed_base: u32, seed_offset: u32) -> Self {
        Self {
            seed_base,
            seed_offset,
            init_base: 0x682f0161,
            init_offset: 0x46790905,
            prime: 4294967291,
            intermediate_xor: 0x5bf03635,
        }
    }
}

impl RandomSequenceBuilder<u64> {
    pub fn new(seed_base: u64, seed_offset: u64) -> Self {
        Self {
            seed_base,
            seed_offset,
            init_base: 0x682f01615bf03635,
            init_offset: 0x46790905682f0161,
            prime: 18446744073709551427, // largest prime: 18446744073709551557
            intermediate_xor: 0x5bf0363546790905,
        }
    }
}

pub trait QuadraticResidue {
    fn residue(self, prime: Self) -> Self;
}

macro_rules! impl_residue {
    ($base_type:ident, $larger_type:ident) => {
        impl QuadraticResidue for $base_type {
            /// Compute the quadratic residue of this number against a prime.
            fn residue(self, prime: Self) -> Self {
                ((self as $larger_type * self as $larger_type) % prime as $larger_type) as Self
            }
        }
    };
}

impl_residue!(u8, u16);
impl_residue!(u16, u32);
impl_residue!(u32, u64);
impl_residue!(u64, u128);

#[cfg(test)]
mod tests {
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;
    use std::string::ToString;

    use super::*;

    /// Check the prime satisfies `p = 3 mod 4`.
    fn is_3_mod_4(n: u64) -> bool {
        n % 4 == 3
    }

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    macro_rules! test_config {
        ($name:ident, $type:ident, $check:literal) => {
            #[test]
            fn $name() {
                let config = RandomSequenceBuilder::<$type>::new(0, 0);
                let config_orig = config.clone();

                // check the configured prime number satisfies the requirements
                // is_prime crate makes this very quick for u64
                let is_prime_res = is_prime::is_prime(&config.prime.to_string());
                let is_3_mod_4_res = is_3_mod_4(config.prime as u64);

                let mut found_number = config.prime;
                if !is_prime_res || !is_3_mod_4_res {
                    if found_number % 2 == 0 {
                        found_number -= 1;
                    }

                    // suggest a suitable prime number (slow, but only run when there's a bad prime)
                    let mut found_next_prime = false;
                    let mut found_next_3_mod_4 = false;

                    while !found_next_prime || !found_next_3_mod_4 {
                        found_number -= 2;
                        found_next_prime = is_prime::is_prime(&found_number.to_string());
                        found_next_3_mod_4 = is_3_mod_4(found_number as u64);
                    }
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
            }
        };
    }

    test_config!(test_u8_config, u8, 256);
    test_config!(test_u16_config, u16, 65536);
    test_config!(test_u32_config, u32, 100_000);
    test_config!(test_u64_config, u64, 100_000);
}
