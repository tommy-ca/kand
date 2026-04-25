#[cfg(test)]
mod tests {
    use crate::ta::ohlcv::vegas::vegas_arrow;
    use crate::ta::types::TAArrowArray;
    use arrow::array::Array;

    #[test]
    fn debug_vegas() {
        let input_price = vec![100.0; 700];
        let input_arrow = TAArrowArray::from(input_price);
        let (upper, _, _, _) = vegas_arrow(&input_arrow).unwrap();
        println!("upper.value(0) = {}", upper.value(0));
        println!("is_nan = {}", upper.value(0).is_nan());
    }
}
