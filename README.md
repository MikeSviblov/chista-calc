# calc

[Русский](README.ru.md) · **English**

A modern equivalent of "Chista Calculator 2.0" ("Чиста калькулятор 2.0"), rewritten in Rust.

The original was a Windows desktop app (Delphi, the `TReckoner` engine, DCPcrypt crypto
backend); its sources are lost. The inventory of functions and syntax was recovered by
reverse-engineering the binary — see `reverse/extracted-inventory.md` and the design doc
`docs/superpowers/specs/2026-07-02-modern-calc-design.md`. This is not a 1:1 clone but a
modern equivalent: syntax/API chosen freely, with no backward compatibility with the old
script files.

## Where it runs

Two frontends over a shared core:

- **`calc`** — a cross-platform CLI, a single binary (Linux/macOS/Windows — anywhere Rust
  builds).
- **`calc-notepad`** — a desktop GUI notepad (egui/eframe), Linux and Windows.

No daemons, no network, no runtime dependencies.

## Architecture

A Cargo workspace of three crates:

- **`calc-core`** — the engine library, with no UI I/O: `lexer` → `parser` → `ast` →
  `eval`, plus `env` (scopes: variables, user functions, aliases) and
  `registry`/`builtins` (built-in functions). The core is reusable for future GUI/web
  wrappers.
- **`calc-cli`** — a thin binary: REPL, running script files (`--file`), a one-off
  expression from a command-line argument.
- **`calc-notepad`** — the GUI notepad (egui/eframe): a live inline calculator over the
  same core. See the "GUI notepad" section.

## Building

```sh
cargo build --release
```

The binary appears at `target/release/calc`.

## Using the CLI (`calc`)

Everything below is about the console binary `calc` (terminal commands). Working in the
GUI notepad is described separately in the [GUI notepad](#gui-notepad) section.

Four modes of operation.

### One-off expression

```sh
calc "2 + 2 * 3"
# 8

calc "IntToRoman(2024)"
# MMXXIV

calc 'Sha256("abc")'
# ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
```

### REPL

```sh
calc
```

Starts the interactive mode (`> ` prompt). Exit with `Ctrl-D`.

### Script from a file

```sh
calc --file script.calc
```

Example `script.calc`:

```
x = 6
fn sq(n) = n * n
i = 0
result = 0
while (i < x) {
    result = result + sq(i)
    i = i + 1
}
print(result)
```

In `--file` mode the only output comes from explicit `print(...)`; the value of the last
statement is not printed automatically. A one-off expression (`calc "expr"`) and the
REPL, by contrast, print the computed value.

### Function help

```sh
calc help            # list all functions by category
calc help Sqrt       # bilingual article for a function: signature, RU/EN, example
```

The same `help` and `help <name>` commands work inside the REPL too.

### Message language

Errors, `help` output, and the REPL prompt are available in Russian and English:

```sh
calc --lang en "1/0"      # Division by zero (position 1)
CALC_LANG=en calc help Sqrt
```

The default is Russian. Priority: the `--lang` flag > the `CALC_LANG` variable > Russian.

## GUI notepad

`calc-notepad` — "a notepad that computes": a desktop frontend (egui/eframe) over the same
`calc-core`, echoing the idea of the original.

### Launching

```sh
calc-notepad
```

### How to use it

1. Launch `calc-notepad` — a window opens with a few example lines.
2. Type an expression (e.g. `2 + 2 * 3`) and press **Enter** — the result appears on the
   line below, in green.
3. Declare variables and functions as you go: `price = 1990`, then on the next line
   `price * 12` — the assignment is silent, the expression is evaluated. State is visible
   to all lines below.
4. Forgot a function name — start typing and press **Tab**: a list of matches (with
   signatures) pops up; ↑/↓ and Enter to pick. Or open **Help** and click **Try** on an
   example.
5. Edit the text like an ordinary notepad (arrows, selection, copy/paste); each line is
   recomputed on Enter. An error on a line shows in red and does not affect its neighbours.
6. **Save** the result to `*.calc` (later **Open**); optionally toggle "always on top" and
   the font size on the toolbar. The text is autosaved between runs anyway.

### A single input/output field

Input and output are **one field**, as in the original (not a notepad plus a separate
results column). You write an expression, press **Enter** — the result appears on the line
**below, in green**, right under the expression. It edits like a normal multiline editor:
arrows, cursor, selection, copy/paste — all native.

The logic is per-line and independent:

- assignments and unfinished/syntactically incomplete lines are **silent** (no result);
- only a bare expression yields a green result;
- a runtime error (division by zero, unknown variable) shows in **red**;
- an error on one line does not blank the others; state (variables, functions) accumulates
  top to bottom.

### Help and autocomplete

- **Help** (a toolbar button) — a window with search and a category-grouped function list
  on the left and a bilingual article on the right (signature, RU/EN description, example,
  error notes). The **Try** button inserts the example into the notepad and computes it
  immediately; **Insert** puts in `Name(` for you to fill in manually.
- **Tab autocomplete**: type a name prefix and press **Tab** — a list of matching functions
  (with signatures) pops up at the cursor. ↑/↓ to select, Enter/Tab to insert `Name(`, Esc
  to close. A single match is inserted right away.

### Toolbar and the rest

Open/Save (`*.calc`), Clear, Help, font size, "always on top", and an **RU/EN** language
toggle (button on the right). Switching the language immediately re-renders the error lines;
the choice is saved between runs. There is syntax highlighting and autosave of the text.

### Release binaries

- `calc-notepad-linux-x64` — Linux (x86_64);
- `calc-notepad.exe` — Windows (x86_64), cross-compiled from Linux via mingw-w64.

## Language syntax

- **Numbers**: decimal (`42`, `3.14`), hexadecimal (`0x1F`), binary (`0b1010`), octal
  (`0o17`).
- **Strings**: `"..."`.
- **Variables**: `x = expr`.
- **User functions**: `fn name(params) = expr` (the body is a single expression).
- **Aliases**: `alias new = existing`.
- **Loops**: `while (cond) { ... }`, `repeat N { ... }`.
- **Output**: `print(...)`.
- **Comments**: `# comment` (to the end of the line).
- **Operators** (from weakest to strongest binding):
  `||` → `&&` → `== != < <= > >=` → `+ -` → `* / %` → unary `- !` →
  `^` (right-associative, binds tighter than everything, even unary minus:
  `-2 ^ 2` = `-(2 ^ 2)`).

## Built-in functions

The full list follows the names actually registered in `calc-core/src/builtins/`.

**Math** (`math.rs`): `Abs`, `Ceil`, `E`, `Exp`, `Fact`, `Floor`, `Frac`, `Gcd`,
`Hypot`, `Lcm`, `Ln`, `Log`, `Log10`, `Log2`, `Max`, `Min`, `Pi`, `Pow`, `Round`,
`Sign`, `Sqr`, `Sqrt`, `Trunc`.

**Trigonometry** (`trig.rs`): `Sin`, `Cos`, `Tan`, `Cotan`, `SinH`, `CosH`, `TanH`,
`ArcSin`, `ArcCos`, `ArcTan`, `ArcSinH`, `ArcCosH`, `ArcTanH`, `DegToRad`, `RadToDeg`.

**Number systems** (`bases.rs`): `IntToBase`, `BaseToInt`, `IntToHex`, `HexToInt`,
`IntToBin`, `BinToInt`, `IntToOct`, `OctToInt`, `IntToRoman`, `RomanToInt`. Arbitrary-base
`IntToBase`/`BaseToInt` supports the range 2..36.

**Bits** (`bits.rs`): `And`, `Or`, `Xor`, `Not`, `Shl`, `Shr`, `BitSet`, `BitClear`,
`BitToggle`, `BitTest`.

**Strings** (`strings.rs`): `Length`, `Concat`, `Copy`, `Pos`, `Replace`, `Upper`,
`Lower`, `Trim`, `TrimLeft`, `TrimRight`, `Reverse`, `Compare`, `Chr`, `Ord`.

**Hashes** (`hash.rs`): the dispatcher `Hash(alg, data)` (algorithms: `md5`, `sha1`,
`sha224`, `sha256`, `sha384`, `sha512`, `sha3_256`, `sha3_512`, `ripemd160`, `tiger`,
`crc32`, `adler32`, case-insensitive) plus direct alias functions `Md5`, `Sha1`, `Sha224`,
`Sha256`, `Sha384`, `Sha512`, `Sha3_256`, `Sha3_512`, `RipeMD160`, `Tiger`, `Crc32`,
`Adler32`.

**Ciphers** (`cipher.rs`): `Encrypt(alg, keyHex, data)`, `Decrypt(alg, keyHex, dataHex)`.

**Files** (`fileio.rs`): `FileToStr`, `StrToFile`, `AppendFile`.

**Date & time** (`datetime.rs`): `Now`, `FormatFloat`.

**Output**: `print`.

## Known limitations (v1)

- **Numbers**: `i128` integers and `f64` (double) reals; no arbitrary precision (bignum).
- **Ciphers**: only AES-128/192/256-CBC (the algorithm is given as the string `aes` or
  `rijndael` — aliases of the same thing), PKCS7 padding, with a FIXED zero IV —
  deterministic (handy for tests/reproducibility) but **not secure** for real encryption.
  The original's exotic ciphers (Blowfish, DES, Twofish, Serpent, CAST, IDEA, TEA, Ice,
  MARS, Misty1) are not included.
- **Hashes**: MD5, SHA-1, SHA-2 (224/256/384/512), SHA-3 (256/512), RIPEMD-160, Tiger,
  CRC32, Adler32; the original's exotic algorithms (Haval, Gost, etc.) are not included.
- **Guards**: expression nesting depth ≤ 150, user-function recursion depth ≤ 512, loop
  iteration count (`while`/`repeat`) ≤ 1,000,000 — exceeding any returns an error (protection
  against stack overflow and hangs).
- **Date/time**: minimal (`Now`, `FormatFloat`), no full calendar.
- **Files**: `FileToStr`/`StrToFile`/`AppendFile` work with any path within the invoking
  user's permissions — there is no sandbox or path restriction (a local CLI tool).
- **Multiline constructs in the notepad**: the notepad evaluates each input line
  independently (recompute on Enter), so a loop or block spread across several lines
  (`while (c) {` … `}`) does not work in the notepad — only single-line form. For multiline
  scripts use the CLI (`calc --file`). The core itself fully supports multiline loops/blocks.

## Status

v0.2.8. The whole workspace is covered by tests (135 tests: core + notepad), clippy
`-D warnings` is clean. Both frontends are ready — the CLI and the GUI notepad, with an
RU/EN language toggle. exe + Linux builds are published to releases on tag.

## License

MIT.
