use kand::ohlcv::mfi;
use pyo3::prelude::*;

/// Calculates Money Flow Index (MFI) for a price series.
...
///   >>> mfi, typ_prices, money_flows, pos_flows, neg_flows = kand.mfi(high, low, close, volume, 14)
///   ```

// Arrow wrapper
crate::kand_py_arrow_wrapper_multi!(
    mfi_arrow,
    kand::ta::ohlcv::mfi::mfi_arrow,
    inputs: { high, low, close, volume },
    params: { period: usize },
    output_count: 5
);
