use kand::TAFloat;

fn main() {
    let input = vec![
        35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
        35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
        35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
    ];
    let period = 5;
    let alpha = 1.0 / period as TAFloat;

    let mut ema1 = vec![0.0; input.len()];
    ema1[0] = input[0];
    for i in 1..input.len() {
        ema1[i] = input[i] * alpha + ema1[i-1] * (1.0 - alpha);
    }

    let mut ema2 = vec![0.0; input.len()];
    ema2[0] = ema1[0];
    for i in 1..input.len() {
        ema2[i] = ema1[i] * alpha + ema2[i-1] * (1.0 - alpha);
    }

    let mut ema3 = vec![0.0; input.len()];
    ema3[0] = ema2[0];
    for i in 1..input.len() {
        ema3[i] = ema2[i] * alpha + ema3[i-1] * (1.0 - alpha);
    }

    for i in 0..input.len() {
        let tema = 3.0 * ema1[i] - 3.0 * ema2[i] + ema3[i];
        let trix = if i > 0 { (ema3[i] - ema3[i-1]) / ema3[i-1] * 100.0 } else { 0.0 };
        println!("Index {}: TEMA={:.4}, TRIX={:.4}", i, tema, trix);
    }
}
