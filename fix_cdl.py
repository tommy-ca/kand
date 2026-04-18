import os
import re

cdl_files = [f for f in os.listdir("kand/src/ta/ohlcv") if f.startswith("cdl_") and f.endswith(".rs")]

for f in cdl_files:
    path = os.path.join("kand/src/ta/ohlcv", f)
    with open(path, "r") as file:
        content = file.read()
    
    # Fix .into() 
    content = content.replace("Signal::Bullish.into()", "Signal::Bullish as TAInt")
    content = content.replace("Signal::Bearish.into()", "Signal::Bearish as TAInt")
    content = content.replace("Signal::Neutral.into()", "Signal::Neutral as TAInt")
    
    with open(path, "w") as file:
        file.write(content)

