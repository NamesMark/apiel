//! Integration tests verified against Dyalog APL 19.0.

use apiel::parse::{eval_to_val, format_val};
use apiel::{Env, apl};

fn assert_apl(expr: &str, expected: &[f64], desc: &str) {
    let result = apl!(expr).unwrap_or_else(|e| panic!("[{desc}] `{expr}` failed: {e}"));
    assert_eq!(
        result.len(),
        expected.len(),
        "[{desc}] `{expr}`: length mismatch — got {result:?}, expected {expected:?}"
    );
    for (i, (got, exp)) in result.iter().zip(expected).enumerate() {
        assert!(
            (got - exp).abs() < 1e-9,
            "[{desc}] `{expr}` element [{i}]: got {got}, expected {exp}"
        );
    }
}

#[test]
fn reference_tests() {
    let e = std::f64::consts::E;
    let pi = std::f64::consts::PI;

    let cases: &[(&str, &[f64], &str)] = &[
        // Dyadic arithmetic
        ("3 + 4", &[7.0], "dyadic add scalar"),
        ("1 2 3 + 4 5 6", &[5.0, 7.0, 9.0], "dyadic add vector"),
        ("10 - 1 2 3", &[9.0, 8.0, 7.0], "dyadic sub scalar-vector"),
        ("1 2 3 × 2 4 6", &[2.0, 8.0, 18.0], "dyadic mul vector"),
        (
            "5 25 125 ÷ 5",
            &[1.0, 5.0, 25.0],
            "dyadic div vector-scalar",
        ),
        ("2 * 10", &[1024.0], "dyadic power scalar"),
        ("1 2 3 * 2 4 6", &[1.0, 16.0, 729.0], "dyadic power vector"),
        ("10 ⍟ 100", &[2.0], "dyadic log"),
        ("3 ⌈ 5", &[5.0], "dyadic max"),
        ("3 ⌊ 5", &[3.0], "dyadic min"),
        ("2 ! 5", &[10.0], "dyadic binomial scalar"),
        (
            "0 1 2 3 4 5 ! 5",
            &[1.0, 5.0, 10.0, 10.0, 5.0, 1.0],
            "dyadic binomial vector",
        ),
        ("3 | 10", &[1.0], "dyadic residue scalar"),
        (
            "5 | 1 2 3 4 5 6 7 8 9 10",
            &[1.0, 2.0, 3.0, 4.0, 0.0, 1.0, 2.0, 3.0, 4.0, 0.0],
            "dyadic residue vector",
        ),
        // Monadic operations
        ("- 1 2 3", &[-1.0, -2.0, -3.0], "monadic negate"),
        ("+ 1 2 3", &[1.0, 2.0, 3.0], "monadic conjugate"),
        ("× 5", &[1.0], "monadic direction positive"),
        ("× (0 - 3)", &[-1.0], "monadic direction negative"),
        ("× 0", &[0.0], "monadic direction zero"),
        ("÷ 2", &[0.5], "monadic reciprocal"),
        ("÷ 4", &[0.25], "monadic reciprocal 4"),
        ("* 1", &[e], "monadic exp"),
        ("○ 1", &[pi], "monadic pi multiple"),
        ("! 5", &[120.0], "monadic factorial 5"),
        ("! 0", &[1.0], "monadic factorial 0"),
        ("| (0 - 5)", &[5.0], "monadic magnitude negative"),
        ("| 3", &[3.0], "monadic magnitude positive"),
        ("⌈ 3.2", &[4.0], "monadic ceiling"),
        ("⌊ 3.8", &[3.0], "monadic floor"),
        ("⍳ 5", &[1.0, 2.0, 3.0, 4.0, 5.0], "monadic iota"),
        ("⍸ 2 0 1", &[1.0, 1.0, 3.0], "monadic where"),
        // Reduce
        ("+/ 1 2 3 4 5", &[15.0], "reduce add"),
        ("×/ 1 2 3 4 5", &[120.0], "reduce multiply"),
        ("-/ 1 2 3 4 5", &[3.0], "reduce subtract (right-fold)"),
        ("÷/ 100 5 2", &[40.0], "reduce divide (right-fold)"),
        ("⌈/ 3 6 9 1", &[9.0], "monadic max reduce"),
        ("⌊/ 5 10 29 1", &[1.0], "monadic min reduce"),
        // Right-to-left / monadic chaining
        (
            "- ⍳ 5",
            &[-1.0, -2.0, -3.0, -4.0, -5.0],
            "chain negate iota",
        ),
        ("⍳ 3 + 2", &[1.0, 2.0, 3.0, 4.0, 5.0], "chain iota add"),
        ("- 3 + 4", &[-7.0], "chain negate add"),
        ("+/ ⍳ 10", &[55.0], "chain reduce iota"),
        ("2 * ⍳ 5", &[2.0, 4.0, 8.0, 16.0, 32.0], "chain power iota"),
        // Recursion / deep nesting
        ("- - 3", &[3.0], "double negate"),
        ("- - - 3", &[-3.0], "triple negate"),
        ("| - 5", &[5.0], "magnitude of negate"),
        ("- | - 5", &[-5.0], "negate magnitude negate"),
        // Right-to-left (no operator precedence)
        ("2 × 3 + 4", &[14.0], "rtl: mul before add"),
        ("2 + 3 × 4", &[14.0], "rtl: add before mul"),
        ("1 + 2 × 3 + 4", &[15.0], "rtl: three ops"),
        ("10 - 3 - 2", &[9.0], "rtl: chained sub"),
        // Parenthesized nesting
        ("(2 + 3) × (4 + 5)", &[45.0], "parens: two groups"),
        ("(1 + 2) × (3 + (4 × 5))", &[69.0], "parens: nested"),
        // Monadic inside dyadic
        ("2 × - 3", &[-6.0], "dyadic with monadic rhs"),
        ("1 + - 2 + 3", &[-4.0], "monadic negate chains into add"),
        // Monadic chains with vectors
        ("- - 1 2 3", &[1.0, 2.0, 3.0], "double negate vector"),
        ("| - 1 2 3", &[1.0, 2.0, 3.0], "magnitude negate vector"),
        // Complex compositions
        (
            "2 × ⍳ 3 + 2",
            &[2.0, 4.0, 6.0, 8.0, 10.0],
            "dyadic into monadic into dyadic",
        ),
        ("+/ 2 × ⍳ 5", &[30.0], "reduce of dyadic of iota"),
        (
            "⍳ ⌈/ 3 1 5 2",
            &[1.0, 2.0, 3.0, 4.0, 5.0],
            "iota of max-reduce",
        ),
        ("1 + +/ 1 2 3", &[7.0], "scalar plus reduce"),
        ("-/ ⍳ 6", &[-3.0], "reduce-sub of iota"),
        ("! +/ 1 2", &[6.0], "factorial of reduce"),
        // Dyadic ⍳ (Index Of)
        ("1 2 3 4 5 ⍳ 3", &[3.0], "index of: scalar found"),
        ("1 2 3 4 5 ⍳ 6", &[6.0], "index of: scalar not found"),
        (
            "10 20 30 ⍳ 20 40 10",
            &[2.0, 4.0, 1.0],
            "index of: vector mixed",
        ),
        (
            "1 2 3 4 5 ⍳ 3 1 4 1 5",
            &[3.0, 1.0, 4.0, 1.0, 5.0],
            "index of: vector all found",
        ),
        // Dyadic ⍸ (Interval Index)
        (
            "2 4 6 8 ⍸ 3 5 7",
            &[1.0, 2.0, 3.0],
            "interval index: between",
        ),
        (
            "10 20 30 40 ⍸ 15 25 35 45",
            &[1.0, 2.0, 3.0, 4.0],
            "interval index: between tens",
        ),
        (
            "1 3 5 7 ⍸ 0 1 2 3 4 5 6 7 8",
            &[0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0],
            "interval index: full range",
        ),
        // High minus (¯)
        ("¯3", &[-3.0], "high minus scalar"),
        ("¯3 + 5", &[2.0], "high minus in expr"),
        ("1 ¯2 3 ¯4", &[1.0, -2.0, 3.0, -4.0], "high minus in vector"),
        ("¯1 + ¯2", &[-3.0], "high minus both operands"),
        ("2 × ¯3", &[-6.0], "high minus rhs"),
        ("¯3.14", &[-3.14], "high minus float"),
        ("- ¯5", &[5.0], "negate high minus"),
        // Comparison operators
        ("3 = 3", &[1.0], "equal true"),
        ("3 = 4", &[0.0], "equal false"),
        ("1 2 3 = 1 3 3", &[1.0, 0.0, 1.0], "equal vector"),
        ("3 ≠ 4", &[1.0], "not equal true"),
        ("3 ≠ 3", &[0.0], "not equal false"),
        ("3 < 4", &[1.0], "less than true"),
        ("4 < 3", &[0.0], "less than false"),
        ("3 > 4", &[0.0], "greater than false"),
        ("4 > 3", &[1.0], "greater than true"),
        ("3 ≤ 4", &[1.0], "leq less"),
        ("4 ≤ 4", &[1.0], "leq equal"),
        ("5 ≤ 4", &[0.0], "leq greater"),
        ("3 ≥ 4", &[0.0], "geq less"),
        ("4 ≥ 4", &[1.0], "geq equal"),
        ("5 ≥ 4", &[1.0], "geq greater"),
        ("1 2 3 < 2 2 2", &[1.0, 0.0, 0.0], "less than vector"),
        // Shape (monadic ⍴)
        ("⍴ 5", &[], "shape of scalar"),
        ("⍴ 1 2 3", &[3.0], "shape of vector"),
        ("⍴ ⍳ 0", &[0.0], "shape of empty vector"),
        // Reshape (dyadic ⍴)
        ("3 ⍴ 1 2 3 4 5", &[1.0, 2.0, 3.0], "reshape truncate"),
        ("5 ⍴ 1 2", &[1.0, 2.0, 1.0, 2.0, 1.0], "reshape cycle"),
        (
            "2 3 ⍴ ⍳ 6",
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "reshape matrix",
        ),
        (
            "2 3 ⍴ 1",
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            "reshape scalar cycle",
        ),
        ("⍴ 2 3 ⍴ ⍳ 6", &[2.0, 3.0], "shape of reshaped matrix"),
        // Ravel (monadic ,)
        (
            ", 2 3 ⍴ ⍳ 6",
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "ravel matrix",
        ),
        (", 5", &[5.0], "ravel scalar"),
        (", 1 2 3", &[1.0, 2.0, 3.0], "ravel vector"),
        // Catenate (dyadic ,)
        (
            "1 2 3 , 4 5 6",
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "catenate vectors",
        ),
        ("1 , 2 3", &[1.0, 2.0, 3.0], "catenate scalar vector"),
        ("1 2 , 3", &[1.0, 2.0, 3.0], "catenate vector scalar"),
        // Reverse (monadic ⌽)
        ("⌽ 1 2 3 4 5", &[5.0, 4.0, 3.0, 2.0, 1.0], "reverse vector"),
        ("⌽ 5", &[5.0], "reverse scalar"),
        // Rotate (dyadic ⌽)
        ("2 ⌽ 1 2 3 4 5", &[3.0, 4.0, 5.0, 1.0, 2.0], "rotate left 2"),
        (
            "¯1 ⌽ 1 2 3 4 5",
            &[5.0, 1.0, 2.0, 3.0, 4.0],
            "rotate right 1",
        ),
        ("0 ⌽ 1 2 3 4 5", &[1.0, 2.0, 3.0, 4.0, 5.0], "rotate zero"),
        // Transpose (monadic ⍉)
        (
            "⍉ 2 3 ⍴ ⍳ 6",
            &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0],
            "transpose matrix",
        ),
        ("⍴ ⍉ 2 3 ⍴ ⍳ 6", &[3.0, 2.0], "shape of transposed"),
        ("⍉ 1 2 3", &[1.0, 2.0, 3.0], "transpose vector noop"),
        ("⍉ 5", &[5.0], "transpose scalar noop"),
        // Dyadic transpose ⍉
        ("1 2 ⍉ 2 3 ⍴ ⍳ 6", &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], "dyadic transpose: identity perm"),
        ("2 1 ⍉ 2 3 ⍴ ⍳ 6", &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0], "dyadic transpose: swap axes"),
        // Bool ops (∧ ∨ ⍲ ⍱)
        ("1 ∧ 1", &[1.0], "and 1 1"),
        ("1 ∧ 0", &[0.0], "and 1 0"),
        ("0 ∧ 0", &[0.0], "and 0 0"),
        ("1 1 0 0 ∧ 1 0 1 0", &[1.0, 0.0, 0.0, 0.0], "and vector"),
        ("1 ∨ 0", &[1.0], "or 1 0"),
        ("0 ∨ 0", &[0.0], "or 0 0"),
        ("1 1 0 0 ∨ 1 0 1 0", &[1.0, 1.0, 1.0, 0.0], "or vector"),
        ("1 ⍲ 1", &[0.0], "nand 1 1"),
        ("1 ⍲ 0", &[1.0], "nand 1 0"),
        ("0 ⍲ 0", &[1.0], "nand 0 0"),
        ("1 1 0 0 ⍲ 1 0 1 0", &[0.0, 1.0, 1.0, 1.0], "nand vector"),
        ("1 ⍱ 0", &[0.0], "nor 1 0"),
        ("0 ⍱ 0", &[1.0], "nor 0 0"),
        ("1 1 0 0 ⍱ 1 0 1 0", &[0.0, 0.0, 0.0, 1.0], "nor vector"),
        // Replicate (/ dyadic)
        (
            "1 0 1 0 1 / 1 2 3 4 5",
            &[1.0, 3.0, 5.0],
            "replicate filter",
        ),
        ("3 / 7", &[7.0, 7.0, 7.0], "replicate scalar"),
        ("2 1 0 / 1 2 3", &[1.0, 1.0, 2.0], "replicate expand"),
        ("1 0 1 / 10 20 30", &[10.0, 30.0], "replicate select"),
        // Scan (\)
        ("+\\ 1 2 3 4 5", &[1.0, 3.0, 6.0, 10.0, 15.0], "scan add"),
        (
            "×\\ 1 2 3 4 5",
            &[1.0, 2.0, 6.0, 24.0, 120.0],
            "scan multiply",
        ),
        (
            "-\\ 1 2 3 4 5",
            &[1.0, -1.0, 2.0, -2.0, 3.0],
            "scan subtract",
        ),
        ("÷\\ 100 5 2", &[100.0, 20.0, 40.0], "scan divide"),
        // Outer product (∘.)
        (
            "1 2 3 ∘.× 1 2 3",
            &[1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0],
            "outer product multiply",
        ),
        ("⍴ 1 2 3 ∘.× 1 2 3", &[3.0, 3.0], "outer product shape"),
        (
            "1 2 3 ∘.+ 10 20",
            &[11.0, 21.0, 12.0, 22.0, 13.0, 23.0],
            "outer product add",
        ),
        ("⍴ 1 2 3 ∘.+ 10 20", &[3.0, 2.0], "outer product add shape"),
        (
            "1 2 ∘.= 1 2 3",
            &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            "outer product equal",
        ),
        // Take (↑)
        ("3 ↑ 1 2 3 4 5", &[1.0, 2.0, 3.0], "take first 3"),
        ("¯2 ↑ 1 2 3 4 5", &[4.0, 5.0], "take last 2"),
        (
            "7 ↑ 1 2 3",
            &[1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0],
            "take with pad",
        ),
        (
            "¯5 ↑ 1 2 3",
            &[0.0, 0.0, 1.0, 2.0, 3.0],
            "take with left pad",
        ),
        // Drop (↓)
        ("2 ↓ 1 2 3 4 5", &[3.0, 4.0, 5.0], "drop first 2"),
        ("¯2 ↓ 1 2 3 4 5", &[1.0, 2.0, 3.0], "drop last 2"),
        ("0 ↓ 1 2 3", &[1.0, 2.0, 3.0], "drop zero"),
        // Grade up/down (⍋⍒)
        ("⍋ 3 1 4 1 5 9", &[2.0, 4.0, 1.0, 3.0, 5.0, 6.0], "grade up"),
        (
            "⍒ 3 1 4 1 5 9",
            &[6.0, 5.0, 3.0, 1.0, 2.0, 4.0],
            "grade down",
        ),
        (
            "⍋ 5 4 3 2 1",
            &[5.0, 4.0, 3.0, 2.0, 1.0],
            "grade up reversed",
        ),
        // Dfns (monadic)
        ("{⍵+1} 5", &[6.0], "dfn monadic simple"),
        (
            "{⍵×⍵} 1 2 3 4 5",
            &[1.0, 4.0, 9.0, 16.0, 25.0],
            "dfn monadic square",
        ),
        ("{+/⍵} 1 2 3 4 5", &[15.0], "dfn monadic reduce"),
        // Dfns (dyadic)
        ("2 {⍺+⍵} 3", &[5.0], "dfn dyadic add"),
        (
            "10 {⍺×⍵} 1 2 3",
            &[10.0, 20.0, 30.0],
            "dfn dyadic mul vector",
        ),
        ("{⍵ + {⍵×2} 3} 10", &[16.0], "nested dfn scoping"),
        ("5 {⍺ + {⍵×⍵} ⍵} 3", &[14.0], "nested dfn alpha omega"),
        // Guards
        ("{⍵>0: ⍵ ⋄ 0} 5", &[5.0], "guard true"),
        ("{⍵>0: ⍵ ⋄ 0} ¯3", &[0.0], "guard false"),
        (
            "{⍵=0: 100 ⋄ ⍵=1: 200 ⋄ 300} 0",
            &[100.0],
            "multi guard first",
        ),
        (
            "{⍵=0: 100 ⋄ ⍵=1: 200 ⋄ 300} 1",
            &[200.0],
            "multi guard second",
        ),
        (
            "{⍵=0: 100 ⋄ ⍵=1: 200 ⋄ 300} 5",
            &[300.0],
            "multi guard fallback",
        ),
        // Self-reference (∇)
        // Expand (\ dyadic)
        (
            "1 0 1 0 1 \\ 1 2 3",
            &[1.0, 0.0, 2.0, 0.0, 3.0],
            "expand with zeros",
        ),
        (
            "1 1 0 1 \\ 1 2 3",
            &[1.0, 2.0, 0.0, 3.0],
            "expand insert zero",
        ),
        // Circular functions (dyadic ○)
        ("1 ○ 0", &[0.0], "sin 0"),
        ("2 ○ 0", &[1.0], "cos 0"),
        ("3 ○ 1", &[1.5574077246549023], "tan 1"),
        // Expand with repeat counts
        (
            "2 1 2 \\ 1 2 3",
            &[1.0, 1.0, 2.0, 3.0, 3.0],
            "expand repeat counts",
        ),
        // Inner product
        ("1 2 3 +.× 4 5 6", &[32.0], "inner product vector"),
        (
            "⍴ (2 3 ⍴ ⍳ 6) +.× 3 2 ⍴ ⍳ 6",
            &[2.0, 2.0],
            "inner product matrix shape",
        ),
        (
            ", (2 3 ⍴ ⍳ 6) +.× 3 2 ⍴ ⍳ 6",
            &[22.0, 28.0, 49.0, 64.0],
            "inner product matrix data",
        ),
        // First (⊃)
        ("⊃ 1 2 3", &[1.0], "first of vector"),
        ("⊃ 5", &[5.0], "first of scalar"),
        // Unique (∪)
        ("∪ 1 2 3 2 1 4", &[1.0, 2.0, 3.0, 4.0], "unique"),
        // Union (dyadic ∪)
        ("1 2 3 ∪ 3 4 5", &[1.0, 2.0, 3.0, 4.0, 5.0], "union"),
        // Intersection (∩)
        ("1 2 3 ∩ 2 3 4", &[2.0, 3.0], "intersection"),
        // Without (~)
        ("1 2 3 4 5 ~ 2 4", &[1.0, 3.0, 5.0], "without"),
        // Not (monadic ~)
        ("~ 0 1 1 0", &[1.0, 0.0, 0.0, 1.0], "not"),
        // Decode (⊥)
        ("2 ⊥ 1 0 1", &[5.0], "decode binary"),
        ("10 ⊥ 1 2 3", &[123.0], "decode decimal"),
        // Encode (⊤)
        ("2 2 2 ⊤ 5", &[1.0, 0.0, 1.0], "encode binary"),
        ("10 10 10 ⊤ 123", &[1.0, 2.0, 3.0], "encode decimal"),
        // Index (⌷)
        ("2 ⌷ 10 20 30 40 50", &[20.0], "index scalar"),
        (
            "1 3 5 ⌷ 10 20 30 40 50",
            &[10.0, 30.0, 50.0],
            "index vector",
        ),
        // Identity functions ⊣ ⊢
        ("⊢ 42", &[42.0], "monadic right tack (identity)"),
        ("⊢ 1 2 3", &[1.0, 2.0, 3.0], "monadic right tack vector"),
        ("⊣ 42", &[42.0], "monadic left tack (identity)"),
        ("⊣ 1 2 3", &[1.0, 2.0, 3.0], "monadic left tack vector"),
        ("5 ⊢ 42", &[42.0], "dyadic right tack returns right"),
        ("5 ⊣ 42", &[5.0], "dyadic left tack returns left"),
        ("1 2 3 ⊢ 4 5 6", &[4.0, 5.0, 6.0], "dyadic right tack vectors"),
        ("1 2 3 ⊣ 4 5 6", &[1.0, 2.0, 3.0], "dyadic left tack vectors"),
        // Tally ≢
        ("≢ 1 2 3 4 5", &[5.0], "tally of vector"),
        ("≢ 42", &[1.0], "tally of scalar"),
        // Find ⍷
        ("2 3 ⍷ 1 2 3 4 5", &[0.0, 1.0, 0.0, 0.0, 0.0], "find subsequence"),
        ("5 ⍷ 1 2 3 4 5", &[0.0, 0.0, 0.0, 0.0, 1.0], "find single element"),
        ("3 4 5 ⍷ 1 2 3 4 5", &[0.0, 0.0, 1.0, 0.0, 0.0], "find at end"),
        ("9 ⍷ 1 2 3", &[0.0, 0.0, 0.0], "find missing element"),
        // Extended operator reductions
        ("∧/ 1 1 1 0", &[0.0], "and-reduce"),
        ("∧/ 1 1 1 1", &[1.0], "and-reduce all true"),
        ("∨/ 0 0 1 0", &[1.0], "or-reduce (any)"),
        ("∨/ 0 0 0 0", &[0.0], "or-reduce all false"),
        ("≠/ 1 0 1 1", &[1.0], "neq-reduce (parity)"),
        ("1 2 3 ∘.≤ 1 2 3", &[1.0,1.0,1.0, 0.0,1.0,1.0, 0.0,0.0,1.0], "outer product leq"),
        // Commute ⍨
        ("+⍨ 3", &[6.0], "selfie: 3+3"),
        ("+⍨ 1 2 3", &[2.0, 4.0, 6.0], "selfie vector: double"),
        ("×⍨ 1 2 3", &[1.0, 4.0, 9.0], "selfie: square"),
        ("2 -⍨ 5", &[3.0], "commute: 5-2"),
        ("3 ÷⍨ 12", &[4.0], "commute: 12÷3"),
        ("2 *⍨ 3", &[9.0], "commute: 3*2=9"),
        // Power operator ⍣
        ("{⍵+1}⍣3 ⍳ 5", &[4.0, 5.0, 6.0, 7.0, 8.0], "power: increment 3 times"),
        ("{⍵×2}⍣4 (1)", &[16.0], "power: double 4 times"),
        ("{⍵+1}⍣0 (5)", &[5.0], "power 0 is identity"),
        // Compose ∘
        ("{⍵+1}∘{⍵×2} 3", &[7.0], "compose: (3×2)+1 = 7"),
        ("{⍵×⍵}∘{⍵+1} 4", &[25.0], "compose: (4+1)² = 25"),
        // Function trains — fork (3-train)
        ("(+/ ÷ ≢) 2 4 6 8 10", &[6.0], "fork: average"),
        ("(⌈/ - ⌊/) 3 1 4 1 5 9", &[8.0], "fork: range = max-min"),
        ("(+/ ÷ ≢) 10 20 30", &[20.0], "fork: average of 3"),
        // Function trains — atop (2-train)
        ("(- ×) 3", &[-1.0], "atop: negate(signum(3))"),
        ("(- ×) ¯5", &[1.0], "atop: negate(signum(-5))"),
        ("(⌊ ÷) 7", &[0.0], "atop: floor(reciprocal(7))"),
        // Rank operator ⍤
        ("{+/⍵}⍤1 ⊢ 2 3 ⍴ ⍳ 6", &[6.0, 15.0], "rank 1: sum each row"),
        ("{⌽⍵}⍤1 ⊢ 2 3 ⍴ ⍳ 6", &[3.0, 2.0, 1.0, 6.0, 5.0, 4.0], "rank 1: reverse each row"),
        // Over ⍥
        ("{⍵×2}⍥{⍵+1} 5", &[12.0], "over monadic: (5+1)×2 = 12"),
        ("3 {⍺+⍵}⍥{⍵×⍵} 4", &[25.0], "over dyadic: 3²+4² = 25"),
        // At operator @
        ("{⍵×10}@(2 3) ⊢ ⍳ 5", &[1.0, 20.0, 30.0, 4.0, 5.0], "at: multiply at indices"),
        ("{0}@(1 3 5) ⊢ ⍳ 5", &[0.0, 2.0, 0.0, 4.0, 0.0], "at: replace at indices"),
        // Key operator ⌸
        ("{≢⍵}⌸ 1 1 2 3 3 3", &[2.0, 1.0, 3.0], "key: count each group"),
        ("{⍺}⌸ 1 1 2 3 3 3", &[1.0, 2.0, 3.0], "key: unique keys"),
    ];

    let mut failures = Vec::new();
    for (expr, expected, desc) in cases {
        let result = std::panic::catch_unwind(|| assert_apl(expr, expected, desc));
        if let Err(e) = result {
            let msg = e
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("unknown panic");
            failures.push(format!("  FAIL: {desc}\n        {msg}"));
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n{} of {} Dyalog reference tests failed:\n{}\n",
            failures.len(),
            cases.len(),
            failures.join("\n")
        );
    }
}

fn assert_apl_env(expr: &str, env: &mut Env, expected: &[f64], desc: &str) {
    let result = apl!(expr, env).unwrap_or_else(|e| panic!("[{desc}] `{expr}` failed: {e}"));
    assert_eq!(
        result.len(),
        expected.len(),
        "[{desc}] `{expr}`: length mismatch — got {result:?}, expected {expected:?}"
    );
    for (i, (got, exp)) in result.iter().zip(expected).enumerate() {
        assert!(
            (got - exp).abs() < 1e-9,
            "[{desc}] `{expr}` element [{i}]: got {got}, expected {exp}"
        );
    }
}

#[test]
fn variables_and_assignment() {
    let mut env = Env::new();

    assert_apl_env("a←5", &mut env, &[5.0], "assign scalar");
    assert_apl_env("a", &mut env, &[5.0], "read scalar");
    assert_apl_env("a + 3", &mut env, &[8.0], "use in expr");
    assert_apl_env("b←1 2 3", &mut env, &[1.0, 2.0, 3.0], "assign vector");
    assert_apl_env("a × b", &mut env, &[5.0, 10.0, 15.0], "scalar times vector");
    assert_apl_env("c←a+10", &mut env, &[15.0], "assign computed");
    assert_apl_env(
        "⍳ a",
        &mut env,
        &[1.0, 2.0, 3.0, 4.0, 5.0],
        "iota of variable",
    );

    // Named functions
    assert_apl_env("double←{⍵×2}", &mut env, &[0.0], "assign dfn");
    assert_apl_env("double 5", &mut env, &[10.0], "call named monadic");
    assert_apl_env(
        "double 1 2 3",
        &mut env,
        &[2.0, 4.0, 6.0],
        "call named monadic vector",
    );
    assert_apl_env("add←{⍺+⍵}", &mut env, &[0.0], "assign dyadic dfn");
    assert_apl_env("10 add 20", &mut env, &[30.0], "call named dyadic");
    assert_apl_env(
        "1 2 3 add 4 5 6",
        &mut env,
        &[5.0, 7.0, 9.0],
        "call named dyadic vector",
    );

    // Named functions in trains
    assert_apl_env(
        "(double + ×) 3",
        &mut env,
        &[7.0],
        "train with named fn: (double 3)+(× 3) = 6+1 = 7",
    );
    assert_apl_env(
        "(double - double) 5",
        &mut env,
        &[0.0],
        "train with two named fns: (double 5)-(double 5) = 0",
    );
}

#[test]
fn macro_omega_alpha() {
    // Monadic: pass ⍵ from Rust
    let result = apl!("⍵ + 1", omega: &[1.0, 2.0, 3.0]).unwrap();
    assert_eq!(result, vec![2.0, 3.0, 4.0]);

    // Dyadic: pass ⍺ and ⍵ from Rust
    let result = apl!("⍺ × ⍵", alpha: &[10.0], omega: &[1.0, 2.0, 3.0]).unwrap();
    assert_eq!(result, vec![10.0, 20.0, 30.0]);

    // Complex expression with Rust data
    let result = apl!("+/ ⍵", omega: &[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
    assert_eq!(result, vec![15.0]);
}

#[test]
fn recursive_dfns() {
    // Needs larger stack due to deep recursion with env cloning
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            assert_apl("{⍵≤1: ⍵ ⋄ ⍵×∇ ⍵-1} 5", &[120.0], "recursive factorial");
            assert_apl("{⍵≤0: 0 ⋄ ⍵+∇ ⍵-1} 10", &[55.0], "recursive sum");
            assert_apl(
                "{⍵<2: ⍵ ⋄ (∇ ⍵-1)+∇ ⍵-2} 10",
                &[55.0],
                "recursive fibonacci",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn strings_and_chars() {
    use apiel::parse::{eval_to_val, format_val};
    let mut env = Env::new();

    let val = eval_to_val("'hello'", &mut env).unwrap();
    assert_eq!(format_val(&val), "hello");
    assert_eq!(val.shape, vec![5]);

    let val = eval_to_val("⌽ 'hello'", &mut env).unwrap();
    assert_eq!(format_val(&val), "olleh");

    let val = eval_to_val("3 ↑ 'hello'", &mut env).unwrap();
    assert_eq!(format_val(&val), "hel");

    let val = eval_to_val("'hello' , ' world'", &mut env).unwrap();
    assert_eq!(format_val(&val), "hello world");

    // String equality returns numeric
    assert_apl(
        "'hello' = 'hxllo'",
        &[1.0, 0.0, 1.0, 1.0, 1.0],
        "string equality",
    );
}

#[test]
fn matrix_inverse() {
    let mut env = Env::new();
    let val = eval_to_val("⌹ 1 1 ⍴ 4", &mut env).unwrap();
    let v: f64 = val.data[0].clone().into();
    assert!((v - 0.25).abs() < 1e-9, "1x1 inverse: got {v}");

    // Matrix divide: solve Ax = B
    assert_apl_env("4 ⌹ 1 1 ⍴ 2", &mut env, &[2.0], "matdiv scalar");

    let result = apl!("6 10 ⌹ 2 2 ⍴ 1 2 3 4").unwrap();
    assert!((result[0] - -2.0).abs() < 1e-9, "matdiv vec [0]");
    assert!((result[1] - 4.0).abs() < 1e-9, "matdiv vec [1]");
}

#[test]
fn nested_arrays() {
    let mut env = Env::new();

    // Enclose wraps as nested scalar
    let val = eval_to_val("⊂ 1 2 3", &mut env).unwrap();
    assert!(val.is_scalar(), "enclosed should be scalar");
    assert_eq!(format_val(&val), "(1 2 3)");

    // Shape of enclosed is empty (scalar)
    assert_apl("⍴ ⊂ 1 2 3", &[], "shape of enclosed");

    // Disclose unwraps
    assert_apl("⊃ ⊂ 1 2 3", &[1.0, 2.0, 3.0], "disclose enclosed");

    // First of plain vector
    assert_apl("⊃ 1 2 3", &[1.0], "first of vector");

    // Partition
    let val = eval_to_val("1 1 0 1 1 ⊆ 10 20 30 40 50", &mut env).unwrap();
    assert_eq!(format_val(&val), "(10 20) (40 50)");
    assert_eq!(val.data.len(), 2); // two groups

    // Each
    assert_apl(
        "+/¨ (⊂ 1 2 3) , (⊂ 4 5) , (⊂ 6)",
        &[6.0, 9.0, 6.0],
        "reduce each",
    );
    assert_apl("1 +¨ 1 2 3", &[2.0, 3.0, 4.0], "dyadic each");

    let val = eval_to_val("⍳¨ 3 4 5", &mut env).unwrap();
    assert_eq!(val.data.len(), 3);
    assert_eq!(format_val(&val), "(1 2 3) (1 2 3 4) (1 2 3 4 5)");

    let val = eval_to_val("⌽¨ (⊂ 1 2 3) , (⊂ 4 5) , (⊂ 6)", &mut env).unwrap();
    assert_eq!(format_val(&val), "(3 2 1) (5 4) (6)");
}

#[test]
fn depth_and_match() {
    let mut env = Env::new();

    let val = eval_to_val("≡ 42", &mut env).unwrap();
    assert_eq!(format_val(&val), "0", "depth of scalar");

    let val = eval_to_val("≡ 1 2 3", &mut env).unwrap();
    assert_eq!(format_val(&val), "1", "depth of flat vector");

    let val = eval_to_val("≡ ⊂ 1 2 3", &mut env).unwrap();
    assert_eq!(format_val(&val), "2", "depth of enclosed vector");

    let val = eval_to_val("1 2 3 ≡ 1 2 3", &mut env).unwrap();
    assert_eq!(format_val(&val), "1", "match identical vectors");

    let val = eval_to_val("1 2 3 ≡ 1 2 4", &mut env).unwrap();
    assert_eq!(format_val(&val), "0", "match different vectors");

    let val = eval_to_val("1 2 3 ≢ 1 2 4", &mut env).unwrap();
    assert_eq!(format_val(&val), "1", "not match different vectors");

    let val = eval_to_val("1 2 3 ≢ 1 2 3", &mut env).unwrap();
    assert_eq!(format_val(&val), "0", "not match identical vectors");

    let val = eval_to_val("(2 3 ⍴ ⍳ 6) ≡ 1 2 3 4 5 6", &mut env).unwrap();
    assert_eq!(format_val(&val), "0", "match: different shapes");
}

#[test]
fn mix_and_split() {
    let mut env = Env::new();

    // Split: matrix -> nested vector of rows
    let val = eval_to_val("↓ 2 3 ⍴ ⍳ 6", &mut env).unwrap();
    assert_eq!(val.data.len(), 2, "split 2x3 gives 2 elements");
    assert_eq!(format_val(&val), "(1 2 3) (4 5 6)");

    // Mix: nested vector -> matrix
    let val = eval_to_val("↑ (⊂ 1 2 3),(⊂ 4 5 6)", &mut env).unwrap();
    assert_eq!(val.shape, vec![2, 3], "mix produces 2x3 matrix");
    assert_eq!(format_val(&val), "1 2 3 4 5 6");

    // Split then mix is identity (for regular matrix)
    let val = eval_to_val("↑ ↓ 2 3 ⍴ ⍳ 6", &mut env).unwrap();
    assert_eq!(val.shape, vec![2, 3], "split then mix roundtrip");
}

#[test]
fn partitioned_enclose() {
    let mut env = Env::new();

    let val = eval_to_val("1 0 1 0 0 ⊂ 1 2 3 4 5", &mut env).unwrap();
    assert_eq!(format_val(&val), "(1 2) (3 4 5)", "partition: two groups");

    let val = eval_to_val("1 1 1 ⊂ 10 20 30", &mut env).unwrap();
    assert_eq!(format_val(&val), "(10) (20) (30)", "partition: each element");

    let val = eval_to_val("1 0 0 0 0 ⊂ 1 2 3 4 5", &mut env).unwrap();
    assert_eq!(format_val(&val), "(1 2 3 4 5)", "partition: single group");
}

#[test]
fn modified_assignment() {
    let mut env = Env::new();
    assert_apl_env("x←5", &mut env, &[5.0], "assign x");
    assert_apl_env("x+←3", &mut env, &[8.0], "modified assign x+←3");
    assert_apl_env("x", &mut env, &[8.0], "x is now 8");
    assert_apl_env("x×←2", &mut env, &[16.0], "modified assign x×←2");
    assert_apl_env("x", &mut env, &[16.0], "x is now 16");
}
