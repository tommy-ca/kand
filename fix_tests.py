import os
import re

files_to_add_import = [
    "ad.rs", "adosc.rs", "adr.rs", "adxr.rs", "bbands.rs", "bop.rs", "cci.rs", "dema.rs", "mfi.rs", "minus_dm.rs", 
    "obv.rs", "plus_dm.rs", "sma.rs", "t3.rs", "tema.rs", "trange.rs", "trima.rs", "typprice.rs", "willr.rs", "wma.rs", 
    "correl.rs", "max.rs", "min.rs", "stddev.rs", "sum.rs"
]

base_dirs = ["kand/src/ta/ohlcv", "kand/src/ta/stats"]

for d in base_dirs:
    for f in os.listdir(d):
        if not f.endswith(".rs"): continue
        path = os.path.join(d, f)
        
        with open(path, "r") as file:
            content = file.read()
            
        if f in files_to_add_import:
            # Add `use crate::ta::types::TAArrowArray;` to mod tests if missing
            if "mod tests {" in content and "TAArrowArray;" not in content:
                content = content.replace("mod tests {", "mod tests {\n    use crate::ta::types::TAArrowArray;")
            
        # Fix .value(x).is_null() to .is_null(x)
        content = re.sub(r'(\w+)\.value\(([^)]+)\)\.is_null\(\)', r'\1.is_null(\2)', content)
        
        # Fix expected.is_null() to expected.is_nan()
        content = re.sub(r'(\w+)\.is_null\(\)', lambda m: m.group(1) + ".is_nan()" if m.group(1) in ["expected", "value"] else m.group(0), content)
        
        # Fix array[i].is_null() to array[i].is_nan() if array is not an arrow array
        content = re.sub(r'([a-zA-Z0-9_]+)\[([^\]]+)\]\.is_null\(\)', r'\1[\2].is_nan()', content)
        
        # Fix .is_nan(0) to .is_null(0) 
        if "is_nan" in content and f == "sma.rs":
            content = content.replace("is_nan(", "is_null(")

        with open(path, "w") as file:
            file.write(content)

