#!/usr/bin/env python3
import re

with open('/workspace/scripts/configure_security.sh', 'r') as f:
    lines = f.readlines()

in_single_quote = False
in_double_quote = False
line_num = 0

for line_num, line in enumerate(lines, 1):
    i = 0
    while i < len(line):
        char = line[i]
        next_char = line[i+1] if i+1 < len(line) else ''
        
        # Handle escape sequences
        if char == '\\' and i+1 < len(line):
            i += 2
            continue
            
        # Handle quotes
        if char == '"' and not in_single_quote:
            in_double_quote = not in_double_quote
            if in_double_quote:
                print(f"Line {line_num}: Opening double quote at column {i+1}: {line.strip()}")
            else:
                print(f"Line {line_num}: Closing double quote at column {i+1}: {line.strip()}")
        elif char == "'" and not in_double_quote:
            in_single_quote = not in_single_quote
            if in_single_quote:
                print(f"Line {line_num}: Opening single quote at column {i+1}: {line.strip()}")
            else:
                print(f"Line {line_num}: Closing single quote at column {i+1}: {line.strip()}")
                
        i += 1

print(f"\nAt end of file:")
print(f"In double quote: {in_double_quote}")
print(f"In single quote: {in_single_quote}")
