import os
import subprocess

def run_cmd(cmd):
    result = subprocess.run(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if result.returncode != 0:
        return None
    return result.stdout

def restore():
    for d in ['kand-py/src/ta/ohlcv', 'kand-py/src/ta/stats']:
        for f in os.listdir(d):
            if not f.endswith('.rs') or f == 'mod.rs' or f == 'macros.rs':
                continue
            path = os.path.join(d, f)
            with open(path, 'r') as file:
                current_content = file.read()
            
            arrow_idx = current_content.find('// Arrow wrapper')
            arrow_part = ""
            if arrow_idx != -1:
                arrow_part = current_content[arrow_idx:]
            
            main_content = run_cmd(f"git show main:{path}")
            if main_content is None:
                print(f"Skipping {path} (not in main)")
                continue
                
            new_content = main_content + "\n" + arrow_part
            with open(path, 'w') as file:
                file.write(new_content)
                
            print(f"Restored {path}")

if __name__ == '__main__':
    restore()
