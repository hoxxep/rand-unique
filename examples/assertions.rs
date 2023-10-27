use std::collections::HashMap;
use rand_sequence::RandomSequenceBuilder;

/// Checking what permutations are delivered for small sequences, as we would like each seed to
/// produce a unique sequence.
fn main() {
    for length in (1..=6).rev() {
        for seed in 0..10 {
            let values: Vec<usize> = RandomSequenceBuilder::<usize>::seed(seed)
                .with_max(length - 1)
                .into_iter()
                .take(length)
                .collect();

            println!("assert_sequence!({}, vec!{:?});", seed, &values);
        }

        println!();
    }

    // check how many permutations are delivered for small sequences
    // TODO: make significant improvements to the number of permutations we produce.
    println!("Reporting how many permutations we can produce. Ideally seen=opt.");
    for length in [u16::MAX as usize, u8::MAX as usize].into_iter().chain((1..=15usize).rev()) {
        let mut total_permutations: usize = 362880;
        if length < 15 {
            total_permutations = std::cmp::min((1..=length).product(), total_permutations);  // factorial: length!
        }

        let mut seen_permutations = HashMap::new();
        for seed in 0..total_permutations {
            let values: Vec<usize> = RandomSequenceBuilder::<usize>::seed(seed as u64)
                .with_max(length - 1)
                .into_iter()
                .take(std::cmp::min(length, 256))
                .collect();

            *seen_permutations.entry(values).or_insert(0) += 1;
        }

        println!("len={} opt={} seen={}", length, total_permutations, seen_permutations.len());
    }
}
