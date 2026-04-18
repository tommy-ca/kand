use kand::ta::types::MAType as KandMAType;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MAType {
    DEMA = 0,
    EMA = 1,
    KAMA = 2,
    MAMA = 3,
    RMA = 4,
    SMA = 5,
    T3 = 6,
    TEMA = 7,
    TRIMA = 8,
    WMA = 9,
}

impl From<MAType> for KandMAType {
    fn from(ma_type: MAType) -> Self {
        match ma_type {
            MAType::DEMA => KandMAType::DEMA,
            MAType::EMA => KandMAType::EMA,
            MAType::KAMA => KandMAType::KAMA,
            MAType::MAMA => KandMAType::MAMA,
            MAType::RMA => KandMAType::RMA,
            MAType::SMA => KandMAType::SMA,
            MAType::T3 => KandMAType::T3,
            MAType::TEMA => KandMAType::TEMA,
            MAType::TRIMA => KandMAType::TRIMA,
            MAType::WMA => KandMAType::WMA,
        }
    }
}
