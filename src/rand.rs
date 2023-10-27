use rand::RngCore;

use crate::{RandomSequence, RandomSequenceBuilder};

macro_rules! init_rand {
    ($type:ident, $tests:ident) => {
        impl RandomSequenceBuilder<$type> {
            /// Initialise a RandomSequenceBuilder from a random seed.
            pub fn rand(rng: &mut impl RngCore) -> Self {
                Self::seed(rng.next_u64())
            }
        }

        impl RandomSequence<$type> {
            /// Initialise a RandomSequence from a random seed.
            pub fn rand(rng: &mut impl RngCore) -> Self {
                RandomSequenceBuilder::<$type>::rand(rng).into_iter()
            }
        }

        #[cfg(test)]
        mod $tests {
            use rand::rngs::OsRng;

            use super::*;

            #[test]
            fn test_rand() {
                let mut rng = OsRng;
                let config1 = RandomSequenceBuilder::<$type>::rand(&mut rng);
                let config2 = RandomSequenceBuilder::<$type>::rand(&mut rng);
                assert_ne!(config1, config2);

                let mut sequence = RandomSequence::<$type>::rand(&mut rng);
                assert_ne!(sequence.next(), sequence.next());
            }
        }
    };
}

init_rand!(u8, tests_u8);
init_rand!(u16, tests_u16);
init_rand!(u32, tests_u32);
init_rand!(u64, tests_u64);
init_rand!(usize, tests_usize);
