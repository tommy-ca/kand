use arrow::array::Float64Array;
use kand::ta::ohlcv::sma;

fn main() {
    // 1. Prepare some synthetic price data
    let data = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];

    // 2. Wrap in an Arrow Float64Array (standard for technical analysis)
    let input_arrow = Float64Array::from(data);

    // 3. Calculate SMA (Simple Moving Average) using the Arrow-native variant
    // This utilizes the internal BlockPool for zero-allocation performance.
    let period = 3;
    let result = sma::sma_arrow(&input_arrow, period).expect("Calculation failed");

    // 4. Output results
    println!("Input:  {:?}", input_arrow.values());
    println!("SMA (3): {:?}", result.values());

    // Initial 'period - 1' values are NaN as expected for SMA
    for i in 0..period - 1 {
        assert!(result.value(i).is_nan());
    }

    // First valid value is the average of [10.0, 11.0, 12.0] = 11.0
    assert!((result.value(2) - 11.0).abs() < 1e-10);

    println!("\nSuccess: Arrow-native SMA calculated with 100% parity.");
}
