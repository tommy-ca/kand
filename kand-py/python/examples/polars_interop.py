import numpy as np
import polars as pl

import kand

# 1. Create a Polars DataFrame with synthetic price data
df = pl.DataFrame(
    {"close": [10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0]}
)

print("Source DataFrame:")
print(df)

# 2. Extract column as an Arrow array and pass to kand-py
# The '_arrow' variants use the PyCapsule protocol for ZERO-COPY transfer.
# No memory is copied between Polars and kand.
close_array = df["close"].to_arrow()
sma_arrow = kand.sma_arrow(close_array, period=3)

# 3. Convert result back to Polars Series and add to DataFrame
# Again, this is zero-copy.
df = df.with_columns(sma=pl.from_arrow(sma_arrow))

print("\nOptimized DataFrame (Zero-Copy):")
print(df)

# 4. Verification
expected = [float("nan"), float("nan"), 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0]
actual = df["sma"].to_list()

for e, a in zip(expected, actual):
    if np.isnan(e):
        assert np.isnan(a)
    else:
        assert abs(e - a) < 1e-10

print("\nSuccess: Polars-to-Kand zero-copy interop verified.")
