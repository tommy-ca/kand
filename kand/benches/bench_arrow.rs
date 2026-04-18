use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use kand::{TAFloat, ta::ohlcv::sma};
use arrow::array::Float64Array;

fn bench_sma_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA_Comparison");
    let periods = [14, 50, 200];
    let data_sizes = [1_000, 10_000, 100_000];

    for &size in &data_sizes {
        let input: Vec<TAFloat> = (0..size).map(|i| i as TAFloat).collect();
        let input_arrow = Float64Array::from(input.clone());
        let mut output = vec![0.0; size];

        group.throughput(Throughput::Elements(size as u64));

        for &period in &periods {
            // Raw Slice Benchmark (No allocation)
            group.bench_with_input(BenchmarkId::new("Raw_Slice", format!("{}/{}", size, period)), &period, |b, &p| {
                b.iter(|| {
                    sma::sma_raw(&input, p, &mut output);
                });
            });

            // Arrow Benchmark (Includes buffer allocation + NaN filling)
            #[cfg(feature = "arrow")]
            group.bench_with_input(BenchmarkId::new("Arrow_Pooled", format!("{}/{}", size, period)), &period, |b, &p| {
                b.iter(|| {
                    let _ = sma::sma_arrow(&input_arrow, p).unwrap();
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_sma_comparison);
criterion_main!(benches);
