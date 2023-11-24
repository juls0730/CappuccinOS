import rustc_demangle.rustc_demangle as rustc_demangle

def demangle_function_name(mangled_name):
    return rustc_demangle.demangle(mangled_name).get_fn_name(False)

if __name__ == "__main__":
    with open("scripts/symbols.table", 'r') as infile:
        lines = infile.readlines()
        
    sorted_lines = sorted(lines, key=lambda line: int(line.split()[0], 16) if len(line.split()) >= 1 else 0)

    with open("scripts/symbols.table", 'w') as outfile:
        for line in sorted_lines:
            parts = line.split()
            if len(parts) >= 3:
                address = parts[0]
                mangled_name = parts[2]
                demangled_name = demangle_function_name(mangled_name)
                new_line = f"{address} {demangled_name}\n"
                outfile.write(new_line)
