#!/usr/bin/env python3
with open('/workspace/scripts/configure_security.sh', 'r') as f:
    content = f.read()

# Simple check: count quotes
double_quotes = content.count('"')
single_quotes = content.count("'")

print(f"Double quotes: {double_quotes}")
print(f"Single quotes: {single_quotes}")
print(f"Double quotes are {'EVEN' if double_quotes % 2 == 0 else 'ODD'}")
print(f"Single quotes are {'EVEN' if single_quotes % 2 == 0 else 'ODD'}")

# Check if file can be parsed
import subprocess
result = subprocess.run(['bash', '-n', '/workspace/scripts/configure_security.sh'], 
                       capture_output=True, text=True)
print(f"\nbash -n result: {result.returncode}")
if result.stderr:
    print(f"stderr: {result.stderr}")
