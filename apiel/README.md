# apiel

A subset of the [APL programming language](https://en.wikipedia.org/wiki/APL_(programming_language)) implemented in Rust. Evaluate APL expressions from Rust code through the `apl!` macro.

## Usage

```rust
use apiel::apl;

// Evaluate APL expressions
let sum = apl!("+/ ⍳ 10").unwrap();           // [55.0]
let mat = apl!("⍴ 2 3 ⍴ ⍳ 6").unwrap();      // [2.0, 3.0]
let fib = apl!("{⍵<2: ⍵ ⋄ (∇ ⍵-1)+∇ ⍵-2} 10").unwrap();  // [55.0]

// Pass Rust data as ⍵ (right argument)
let result = apl!("+/ ⍵", omega: &[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();  // [15.0]

// Pass both ⍺ (left) and ⍵ (right)
let result = apl!("⍺ × ⍵", alpha: &[10.0], omega: &[1.0, 2.0, 3.0]).unwrap();  // [10.0, 20.0, 30.0]

// Shared environment -- variables persist across calls
let mut env = apiel::Env::new();
apl!("data←⍳ 10", &mut env).unwrap();
apl!("total←+/ data", &mut env).unwrap();
let result = apl!("total", &mut env).unwrap();  // [55.0]

// Define and call named functions
apl!("double←{⍵×2}", &mut env).unwrap();
apl!("double 1 2 3", &mut env).unwrap();  // [2.0, 4.0, 6.0]
```

## What's supported

- **Arithmetic**: `+` `-` `×` `÷` `*` `⍟` `○` `!` `?` `|` `⌈` `⌊` `⌹`
- **Arrays**: `⍴` `,` `⌽` `⍉` `↑` `↓` `⍋` `⍒` `⊂` `⊃` `⊆` `⌷` `∪` `∩` `~`
- **Comparison**: `=` `≠` `<` `>` `≤` `≥` `∧` `∨` `⍲` `⍱`
- **Operators**: `/` (reduce) `\` (scan) `∘.` (outer product) `f.g` (inner product) `¨` (each)
- **Language**: `←` assignment, `{⍵}` dfns, `∇` recursion, `⋄` `:` guards, `¯` high minus, `'...'` strings, `⊥` `⊤` encode/decode

## Affiliation

Capstone project for the [rustcamp](https://github.com/rust-lang-ua/rustcamp) by the [Ukrainian Rust Community](https://www.uarust.com).
