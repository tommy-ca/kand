
import numpy as np

def rma(data, period):
    alpha = 1.0 / period
    output = np.full(len(data), np.nan)
    output[period-1] = np.mean(data[:period])
    for i in range(period, len(data)):
        output[i] = data[i] * alpha + output[i-1] * (1.0 - alpha)
    return output

data = [
    35216.1, 35221.4, 35190.7, 35170.0, 35181.5, 35254.6, 35202.8, 35251.9, 35197.6,
    35184.7, 35175.1, 35229.9, 35212.5, 35160.7, 35090.3, 35041.2, 34999.3, 35013.4,
    35069.0, 35024.6, 34939.5, 34952.6, 35000.0, 35041.8, 35080.0,
]
period = 5

e1 = rma(data, period)
e2 = rma(e1[4:], period) # Not quite right for indexing but let's see values
# To do it properly:
e2_full = np.full(len(data), np.nan)
e2_start = np.mean(e1[4:9])
e2_full[8] = e2_start
for i in range(9, len(data)):
    e2_full[i] = e1[i] * (1.0/5) + e2_full[i-1] * (4.0/5)

e3_full = np.full(len(data), np.nan)
e3_start = np.mean(e2_full[8:13])
e3_full[12] = e3_start
for i in range(13, len(data)):
    e3_full[i] = e2_full[i] * (1.0/5) + e3_full[i-1] * (4.0/5)

trix = np.full(len(data), np.nan)
for i in range(13, len(data)):
    trix[i] = (e3_full[i] - e3_full[i-1]) / e3_full[i-1] * 100.0

print("RMA TRIX last value:", trix[-1])
