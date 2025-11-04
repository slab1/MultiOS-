#!/usr/bin/env python3

with open('/workspace/scripts/configure_security.sh', 'r') as f:
    content = f.read()

# Track quote state
in_double_quote = False
in_single_quote = False
line_num = 1

# Split into lines but keep track of position
for line_num, line in enumerate(content.split('\n'), 1):
    i = 0
    while i < len(line):
        char = line[i]
        
        # Handle escape sequences - skip next character
        if char == '\\' and i + 1 < len(line):
            i += 2
            continue
            
        # Handle quotes
        if char == '"' and not in_single_quote:
            in_double_quote = not in_double_quote
            if in_double_quote:
                print(f"Line {line_num} col {i+1}: OPEN double quote")
            else:
                print(f"Line {line_num} col {i+1}: CLOSE double quote")
        elif char == "'" and not in_double_quote:
            in_single_quote = not in_single_quote
            if in_single_quote:
                print(f"Line {line_num} col {i+1}: OPEN single quote")
            else:
                print(f"Line {line_num} col {i+1}: CLOSE single quote")
                
        i += 1

print(f"\nAt end of file:")
print(f"  In double quote: {in_double_quote}")
print(f"  In single quote: {in_single_quote}")

if in_double_quote or in_single_quote:
    print("\nFOUND UNCLOSED QUOTE!")
