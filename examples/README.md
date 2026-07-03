# Examples

Runnable `.calc` scripts. Run any of them with the CLI:

```sh
calc --file examples/basics.calc
```

Output comes from `print(...)` — in `--file` mode the value of the last statement is
not printed automatically.

| File | Shows |
|------|-------|
| `basics.calc` | variables, formulas, a user function, an alias |
| `functions.calc` | user functions and composition |
| `loops.calc` | `while`/`repeat`: sum, factorial, Fibonacci, GCD, digit sum |
| `number-systems.calc` | Roman numerals, hex/binary, arbitrary base |
| `strings-and-hashes.calc` | string operations and hashes |
| `table.calc` | a multiline loop printing a table |
| `countdown.calc` | a simple countdown loop |

## Notes on the language

- **No `if`** — branch only through a `while` condition; a function body is a single expression.
- **Division is real**: `5 / 2 = 2.5`. Use `Trunc(a / b)` for integer division.
- **Booleans don't mix with arithmetic** (`(x > 0) * 5` is an error).
- **In the GUI notepad**, multiline loops don't work — write the loop on one line with `;`,
  e.g. `s = 0; i = 1; while (i <= 100) { s = s + i; i = i + 1 }; s`. Multiline scripts like the
  ones here are for the CLI (`calc --file`).
