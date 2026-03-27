# apiel

Apiel is a subset of the [APL programming language](https://en.wikipedia.org/wiki/APL_(programming_language)) implemented in Rust.

The project exports a macro (`apl!`) for evaluating APL expressions from Rust code, and a CLI (`apiel-cli`) for interactive use.

## Array Languages

APL was the first language in an "Array programming" or "Iversonian" paradigm. These languages are closer to mathematical notation than to C-like programming languages. The concepts proposed by APL inspired many similar languages, influenced the development of the functional programming paradigm, and had a giant impact on programming as a whole.

## Approach

The project utilizes [Yacc](https://en.wikipedia.org/wiki/Yacc) **(Yet-Another-Compiler-Compiler)** implementation in Rust through [grmtools](https://github.com/softdevteam/grmtools) to build the lexer and parser.

`apiel.l` contains the tokens for the **lexer**, `apiel.y` describes the **Yacc grammar**. The `build.rs` generates Rust code for the lexer and parser. The evaluator in `parse/eval.rs` is a recursive match over the `Expr` AST.

Function trains are handled via token-level preprocessing: parenthesized groups of function references are detected by the lexer and rewritten to dfn expressions before parsing.

## Usage

### CLI

```
cargo run -p apiel-cli
```

or `RUST_LOG=debug cargo run -p apiel-cli` for debugging output.

### Library

```rust
use apiel::apl;

let result = apl!("+/ ⍳ 10").unwrap();  // [55.0]

// Pass data from Rust
let result = apl!("⍺ × ⍵", alpha: &[10.0], omega: &[1.0, 2.0, 3.0]).unwrap();

// Persistent environment
let mut env = apiel::Env::new();
apl!("data←⍳ 10", &mut env).unwrap();
apl!("+/ data", &mut env).unwrap();  // [55.0]
```

## Supported Glyphs and Operations

### Scalar Functions

| Glyph | Monadic operation | Impl. | Dyadic operation | Impl. |
| --- | --- | --- | --- | --- |
| + | Conjugate | ✅* | Addition | ✅ |
| - | Negate | ✅ | Subtraction | ✅ |
| × | Direction (signum) | ✅ | Multiplication | ✅ |
| ÷ | Reciprocal | ✅ | Division | ✅ |
| * | Exponential | ✅ | Power | ✅ |
| ⍟ | Natural logarithm | ✅ | Logarithm | ✅ |
| ○ | Pi multiple | ✅ | Circular functions | ✅ |
| ! | Factorial | ✅ | Binomial | ✅ |
| ? | Roll | ✅ | Deal | ✅ |
| \| | Magnitude | ✅ | Residue | ✅ |
| ⌈ | Ceiling | ✅ | Maximum | ✅ |
| ⌊ | Floor | ✅ | Minimum | ✅ |
| ⌹ | Matrix inverse | ✅ | Matrix divide | ✅ |

\* Not implemented for complex numbers

### Array Functions

| Glyph | Monadic operation | Impl. | Dyadic operation | Impl. |
| --- | --- | --- | --- | --- |
| ⍳ | Index generate | ✅ | Index of | ✅ |
| ⍸ | Where | ✅ | Interval index | ✅ |
| ⍴ | Shape | ✅ | Reshape | ✅ |
| , | Ravel | ✅ | Catenate | ✅ |
| ⌽ | Reverse | ✅ | Rotate | ✅ |
| ⍉ | Transpose | ✅ | Dyadic transpose | ✅ |
| ↑ | Mix | ✅ | Take | ✅ |
| ↓ | Split | ✅ | Drop | ✅ |
| ⍋ | Grade Up | ✅ | - | - |
| ⍒ | Grade Down | ✅ | - | - |
| ⊂ | Enclose | ✅ | Partitioned enclose | ✅ |
| ⊃ | First / Disclose | ✅ | - | - |
| ⊆ | - | - | Partition | ✅ |
| ⌷ | - | - | Index | ✅ |
| ⍷ | - | - | Find | ✅ |

### Selection and Set Functions

| Glyph | Monadic operation | Impl. | Dyadic operation | Impl. |
| --- | --- | --- | --- | --- |
| ∪ | Unique | ✅ | Union | ✅ |
| ∩ | - | - | Intersection | ✅ |
| ~ | Not | ✅ | Without | ✅ |
| ⊣ | Same (identity) | ✅ | Left | ✅ |
| ⊢ | Same (identity) | ✅ | Right | ✅ |
| ≡ | Depth | ✅ | Match | ✅ |
| ≢ | Tally | ✅ | Not Match | ✅ |

### Comparison and Logic

| Glyph | Monadic operation | Impl. | Dyadic operation | Impl. |
| --- | --- | --- | --- | --- |
| = | - | - | Equal | ✅ |
| ≠ | - | - | Not Equal | ✅ |
| < | - | - | Less Than | ✅ |
| > | - | - | Greater Than | ✅ |
| ≤ | - | - | Less or Equal | ✅ |
| ≥ | - | - | Greater or Equal | ✅ |
| ∧ | - | - | And | ✅ |
| ∨ | - | - | Or | ✅ |
| ⍲ | - | - | Nand | ✅ |
| ⍱ | - | - | Nor | ✅ |

### Encoding

| Glyph | Monadic operation | Impl. | Dyadic operation | Impl. |
| --- | --- | --- | --- | --- |
| ⊥ | - | - | Decode | ✅ |
| ⊤ | - | - | Encode | ✅ |

### Operators (Higher-Order)

| Glyph | Name | Impl. | Description |
| --- | --- | --- | --- |
| f/ | Reduce | ✅ | Right fold: `+/ 1 2 3` = 6 |
| f\ | Scan | ✅ | Cumulative fold: `+\ 1 2 3` = `1 3 6` |
| ∘.f | Outer Product | ✅ | All pairs: `1 2 ∘.× 3 4` |
| f.g | Inner Product | ✅ | Generalized matrix multiply |
| f¨ | Each | ✅ | Apply to each element |
| f⍨ | Commute / Selfie | ✅ | `A f⍨ B` = `B f A`; `f⍨ B` = `B f B` |
| f⍣n | Power | ✅ | Apply f n times |
| {f}∘{g} | Compose | ✅ | `f(g(⍵))` |
| {f}⍥{g} | Over | ✅ | Monadic: `f(g(⍵))`; Dyadic: `(g ⍺) f (g ⍵)` |
| {f}⍤k | Rank | ✅ | Apply f to each rank-k cell |
| {f}@i | At | ✅ | Apply f at specified indices |
| {f}⌸ | Key | ✅ | Group-by: apply f to each group |
| (f g h) | Fork (3-train) | ✅ | `(f ⍵) g (h ⍵)` -- e.g. `(+/ ÷ ≢)` for average |
| (f g) | Atop (2-train) | ✅ | `f (g ⍵)` |

Reduce, scan, outer product, inner product, and each work with all 20 primitive operators.

### Language Features

| Feature | Impl. | Description |
| --- | --- | --- |
| ← Assignment | ✅ | Variable binding |
| x+←1 Modified assignment | ✅ | `x←x+1` shorthand, works with all operators |
| x[i]←v Indexed assignment | ✅ | Modify elements at 1-based indices |
| {⍵} Dfns (lambdas) | ✅ | Anonymous functions with `⍵` (right) and `⍺` (left) args |
| f←{⍵} Named functions | ✅ | Store and call functions by name |
| ∇ Self-reference | ✅ | Recursive calls within dfns |
| ⋄ : Guards / Statements | ✅ | Multi-branch conditionals and sequential execution |
| ¯ High minus | ✅ | Negative number literals |
| '...' Strings | ✅ | Character vectors |
| Nested arrays | ✅ | Arrays containing arrays via `⊂` |
| N-dimensional arrays | ✅ | Any rank via `⍴` reshape |
| Scalar extension | ✅ | Auto-broadcast scalars to arrays |

### Examples

```
>>> (+/ ÷ ≢) 2 4 6 8 10
6
>>> (⌈/ - ⌊/) 3 1 4 1 5 9
8
>>> +⍨ 1 2 3
2 4 6
>>> {⍵+1}⍣3 ⍳ 5
4 5 6 7 8
>>> 2 3 ⍴ ⍳ 6
1 2 3 4 5 6
>>> ⍴ 2 3 ⍴ ⍳ 6
2 3
>>> ⌽ 1 2 3 4 5
5 4 3 2 1
>>> 1 2 3 = 1 3 3
1 0 1
>>> {⍵<2: ⍵ ⋄ (∇ ⍵-1)+∇ ⍵-2} 10
55
>>> ∧/ 1 1 1 0
0
>>> {≢⍵}⌸ 1 1 2 3 3 3
2 1 3
```

## Affiliation

This was implemented as my capstone project for the [rustcamp](https://github.com/rust-lang-ua/rustcamp), a Rust bootcamp organized by the Ukrainian Rust Community ([website](https://www.uarust.com), [linked in](https://www.linkedin.com/company/ukrainian-rust-community), [telegram](https://t.me/rustlang_ua), [github](https://github.com/rust-lang-ua), [youtube](https://www.youtube.com/channel/UCmkAFUu2MVOX8ly0LjB6TMA), [twitter](https://twitter.com/rustukraine)).
