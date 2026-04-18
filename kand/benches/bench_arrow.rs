use arrow::array::Float64Array;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kand::{
    TAFloat,
    ta::ohlcv::{cdl_hammer, macd, sma},
};

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
            group.bench_with_input(
                BenchmarkId::new("Raw_Slice", format!("{}/{}", size, period)),
                &period,
                |b, &p| {
                    b.iter(|| {
                        sma::sma_raw(&input, p, &mut output);
                    });
                },
            );

            #[cfg(feature = "arrow")]
            group.bench_with_input(
                BenchmarkId::new("Arrow_Pooled", format!("{}/{}", size, period)),
                &period,
                |b, &p| {
                    b.iter(|| {
                        let _ = sma::sma_arrow(&input_arrow, p).unwrap();
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_macd_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("MACD_Comparison");
    let data_sizes = [1_000, 10_000, 100_000];

    for &size in &data_sizes {
        let input: Vec<TAFloat> = (0..size).map(|i| i as TAFloat).collect();
        let input_arrow = Float64Array::from(input.clone());

        let mut output_macd = vec![0.0; size];
        let mut output_signal = vec![0.0; size];
        let mut output_hist = vec![0.0; size];

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("Raw_Slice", size), &size, |b, _| {
            b.iter(|| {
                macd::macd_raw(
                    &input,
                    12,
                    26,
                    9,
                    &mut output_macd,
                    &mut output_signal,
                    &mut output_hist,
                );
            });
        });

        #[cfg(feature = "arrow")]
        group.bench_with_input(BenchmarkId::new("Arrow_Pooled", size), &size, |b, _| {
            b.iter(|| {
                let _ = macd::macd_arrow(&input_arrow, 12, 26, 9).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_hammer_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hammer_Comparison");
    let data_sizes = [1_000, 10_000, 100_000];

    for &size in &data_sizes {
        let open: Vec<TAFloat> = (0..size).map(|i| i as TAFloat).collect();
        let high: Vec<TAFloat> = (0..size).map(|i| (i + 1) as TAFloat).collect();
        let low: Vec<TAFloat> = (0..size).map(|i| (i - 1) as TAFloat).collect();
        let close: Vec<TAFloat> = (0..size).map(|i| i as TAFloat).collect();

        let open_arrow = Float64Array::from(open.clone());
        let high_arrow = Float64Array::from(high.clone());
        let low_arrow = Float64Array::from(low.clone());
        let close_arrow = Float64Array::from(close.clone());

        let mut output_signals = vec![0; size];
        let mut output_body_avg = vec![0.0; size];

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("Raw_Slice", size), &size, |b, _| {
            b.iter(|| {
                cdl_hammer::cdl_hammer_raw(
                    &open,
                    &high,
                    &low,
                    &close,
                    14,
                    2.0,
                    &mut output_signals,
                    &mut output_body_avg,
                );
            });
        });

        #[cfg(feature = "arrow")]
        group.bench_with_input(BenchmarkId::new("Arrow_Pooled", size), &size, |b, _| {
            b.iter(|| {
                let _ = cdl_hammer::cdl_hammer_arrow(
                    &open_arrow,
                    &high_arrow,
                    &low_arrow,
                    &close_arrow,
                    14,
                    2.0,
                )
                .unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_sma_comparison,
    bench_macd_comparison,
    bench_hammer_comparison
);
criterion_main!(benches);
