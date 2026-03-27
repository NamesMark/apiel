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

## What's Supported

- **Arithmetic**: `+` `-` `×` `÷` `*` `⍟` `○` `!` `?` `|` `⌈` `⌊` `⌹`
- **Arrays**: `⍴` `,` `⌽` `⍉` `↑` `↓` `⍋` `⍒` `⊂` `⊃` `⊆` `⌷` `∪` `∩` `~` `⊣` `⊢` `≡` `≢` `⍷`
- **Comparison**: `=` `≠` `<` `>` `≤` `≥` `∧` `∨` `⍲` `⍱`
- **Operators**: `f/` reduce, `f\` scan, `∘.f` outer product, `f.g` inner product, `f¨` each, `f⍨` commute, `f⍣n` power, `{f}∘{g}` compose, `{f}⍥{g}` over, `{f}⍤k` rank, `{f}@i` at, `{f}⌸` key
- **Trains**: `(f g h)` fork, `(f g)` atop -- supports primitives, reductions, and named functions
- **Language**: `←` assignment, `x+←1` modified assignment, `x[i]←v` indexed assignment, `{⍵}` dfns, `∇` recursion, `⋄` `:` guards, `¯` high minus, `'...'` strings, `⊥` `⊤` encode/decode, nested arrays

## Affiliation

Capstone project for the [rustcamp](https://github.com/rust-lang-ua/rustcamp) by the [Ukrainian Rust Community](https://www.uarust.com).
