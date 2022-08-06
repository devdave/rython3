import tokenize as tk


patterns = {
    "Ignore": tk.Ignore,
    "Number":tk.Number,
    "Pointfloat": tk.Pointfloat,
    "Triple": tk.Triple,
    "String": tk.String,

}

for name, pattern in patterns.items():
    print(name)
    print("="*80)
    print(repr(pattern))
    print()

