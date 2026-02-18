use criterion::{criterion_group, criterion_main};

criterion_group!(benches, sequence::sequence_bench);
criterion_main!(benches);

mod sequence {
    use std::hint::black_box;
    use criterion::{BatchSize, Bencher, Criterion};
    use rand_unique::RandomSequence;

    pub fn sequence_bench(c: &mut Criterion) {
        let mut group = c.benchmark_group("sequence");
        group.bench_function("n_u8", bench_n_u8);
        group.bench_function("n_u16", bench_n_u16);
        group.bench_function("n_u32", bench_n_u32);
        group.bench_function("n_u64", bench_n_u64);
        group.bench_function("rand_u64", bench_rand_u64);
    }

    macro_rules! bench_n {
        ($name:ident, $type:ident) => {
            fn $name(b: &mut Bencher) {
                let sequence = RandomSequence::<$type>::rand(&mut rand::rng());

                b.iter_batched(
                    || rand::random::<$type>(),
                    |index| black_box({ sequence.n(index) }),
                    BatchSize::SmallInput,
                );
            }
        };
    }

    bench_n!(bench_n_u8, u8);
    bench_n!(bench_n_u16, u16);
    bench_n!(bench_n_u32, u32);
    bench_n!(bench_n_u64, u64);

    /// Compare standard random number generation time.
    fn bench_rand_u64(b: &mut Bencher) {
        b.iter(|| black_box(rand::random::<u64>()))
    }
}
