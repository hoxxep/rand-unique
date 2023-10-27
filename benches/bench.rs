use criterion::{criterion_group, criterion_main};

criterion_group!(benches, builder::builder_bench, sequence::sequence_bench);
criterion_main!(benches);

mod builder {
    use criterion::{black_box, BatchSize, Bencher, Criterion};
    use rand::distributions::Distribution;
    use rand::rngs::OsRng;
    use rand_sequence::{RandomSequenceBuilder};

    pub fn builder_bench(c: &mut Criterion) {
        let mut group = c.benchmark_group("builder");
        group.bench_function("find_suitable_prime_small", find_suitable_prime_small);
        group.bench_function("find_suitable_prime_large", find_suitable_prime_large);
    }

    fn find_suitable_prime_small(b: &mut Bencher) {
        let mut rng = OsRng;
        let uniform = rand::distributions::Uniform::new(600, 1000);
        b.iter_batched(
            || uniform.sample(&mut rng),
            |max| black_box(RandomSequenceBuilder::<usize>::find_suitable_prime(max)),
            BatchSize::SmallInput,
        );
    }

    fn find_suitable_prime_large(b: &mut Bencher) {
        let mut rng = OsRng;
        let uniform = rand::distributions::Uniform::new(usize::MAX - 1_000_000, usize::MAX);
        b.iter_batched(
            || uniform.sample(&mut rng),
            |max| black_box(RandomSequenceBuilder::<usize>::find_suitable_prime(max)),
            BatchSize::SmallInput,
        );
    }
}

mod sequence {
    use criterion::{black_box, BatchSize, Bencher, Criterion};
    use rand::rngs::OsRng;
    use rand_sequence::{RandomSequence, RandomSequenceBuilder};

    pub fn sequence_bench(c: &mut Criterion) {
        let mut group = c.benchmark_group("sequence");
        group.bench_function("n_u8", bench_n_u8);
        group.bench_function("n_u16", bench_n_u16);
        group.bench_function("n_u32", bench_n_u32);
        group.bench_function("n_u64", bench_n_u64);
        group.bench_function("n_usize", bench_n_usize);
        group.bench_function("n_usize_max_1000", bench_n_usize_max_1000);
        group.bench_function("rand_u64", bench_rand_u64);
    }

    macro_rules! bench_n {
        ($name:ident, $type:ident) => {
            fn $name(b: &mut Bencher) {
                let sequence = RandomSequence::<$type>::rand(&mut OsRng);

                b.iter_batched(
                    || rand::random::<$type>(),
                    |index| black_box(sequence.n(index)),
                    BatchSize::SmallInput,
                );
            }
        };

        ($name:ident, $type:ident, $max:literal) => {
            fn $name(b: &mut Bencher) {
                let sequence = RandomSequenceBuilder::<$type>::rand(&mut OsRng)
                    .with_max($max)
                    .into_iter();

                b.iter_batched(
                    || rand::random::<$type>(),
                    |index| black_box(sequence.n(index)),
                    BatchSize::SmallInput,
                );
            }
        };
    }

    bench_n!(bench_n_u8, u8);
    bench_n!(bench_n_u16, u16);
    bench_n!(bench_n_u32, u32);
    bench_n!(bench_n_u64, u64);
    bench_n!(bench_n_usize, usize);
    bench_n!(bench_n_usize_max_1000, usize, 1000);

    /// Compare standard random number generation time.
    fn bench_rand_u64(b: &mut Bencher) {
        b.iter(|| black_box(rand::random::<u64>()))
    }
}
