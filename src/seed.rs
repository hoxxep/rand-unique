use crate::builder::RandomSequenceBuilder;
use crate::sequence::RandomSequence;

macro_rules! seed_sequence {
    ($type:ident) => {
        impl RandomSequence<$type> {
            /// Initialise a random sequence from the seeds.
            ///
            /// These seeds should be two uniformly random numbers across the u64 space.
            pub fn new(seed_base: $type, seed_offset: $type) -> Self {
                let config = RandomSequenceBuilder::<$type>::new(seed_base, seed_offset);
                config.into_iter()
            }
        }
    };
}

seed_sequence!(u8);
seed_sequence!(u16);
seed_sequence!(u32);
seed_sequence!(u64);
