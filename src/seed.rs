use crate::builder::RandomSequenceBuilder;
use crate::sequence::RandomSequence;

macro_rules! impl_seed_sequence {
    ($type:ident) => {
        impl RandomSequence<$type> {
            /// Initialise a random sequence from the seeds.
            ///
            /// These seeds should be two uniformly random numbers across the u64 space.
            pub fn seed(seed: u64) -> Self {
                let config = RandomSequenceBuilder::<$type>::seed(seed);
                config.into_iter()
            }
        }
    };
}

impl_seed_sequence!(u8);
impl_seed_sequence!(u16);
impl_seed_sequence!(u32);
impl_seed_sequence!(u64);
impl_seed_sequence!(usize);
