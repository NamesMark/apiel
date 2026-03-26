//! Integration tests verified against Dyalog APL 19.0.

use apiel::{apl, Env};

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
        ("3 + 4",                       &[7.0],                                 "dyadic add scalar"),
        ("1 2 3 + 4 5 6",              &[5.0, 7.0, 9.0],                      "dyadic add vector"),
        ("10 - 1 2 3",                 &[9.0, 8.0, 7.0],                      "dyadic sub scalar-vector"),
        ("1 2 3 × 2 4 6",              &[2.0, 8.0, 18.0],                     "dyadic mul vector"),
        ("5 25 125 ÷ 5",               &[1.0, 5.0, 25.0],                     "dyadic div vector-scalar"),
        ("2 * 10",                      &[1024.0],                              "dyadic power scalar"),
        ("1 2 3 * 2 4 6",              &[1.0, 16.0, 729.0],                   "dyadic power vector"),
        ("10 ⍟ 100",                    &[2.0],                                 "dyadic log"),
        ("3 ⌈ 5",                       &[5.0],                                 "dyadic max"),
        ("3 ⌊ 5",                       &[3.0],                                 "dyadic min"),
        ("2 ! 5",                       &[10.0],                                "dyadic binomial scalar"),
        ("0 1 2 3 4 5 ! 5",            &[1.0, 5.0, 10.0, 10.0, 5.0, 1.0],    "dyadic binomial vector"),
        ("3 | 10",                      &[1.0],                                 "dyadic residue scalar"),
        ("5 | 1 2 3 4 5 6 7 8 9 10",   &[1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,0.0], "dyadic residue vector"),
        // Monadic operations
        ("- 1 2 3",                     &[-1.0, -2.0, -3.0],                   "monadic negate"),
        ("+ 1 2 3",                     &[1.0, 2.0, 3.0],                      "monadic conjugate"),
        ("× 5",                         &[1.0],                                 "monadic direction positive"),
        ("× (0 - 3)",                   &[-1.0],                                "monadic direction negative"),
        ("× 0",                         &[0.0],                                 "monadic direction zero"),
        ("÷ 2",                         &[0.5],                                 "monadic reciprocal"),
        ("÷ 4",                         &[0.25],                                "monadic reciprocal 4"),
        ("* 1",                         &[e],                                   "monadic exp"),
        ("○ 1",                         &[pi],                                  "monadic pi multiple"),
        ("! 5",                         &[120.0],                               "monadic factorial 5"),
        ("! 0",                         &[1.0],                                 "monadic factorial 0"),
        ("| (0 - 5)",                   &[5.0],                                 "monadic magnitude negative"),
        ("| 3",                         &[3.0],                                 "monadic magnitude positive"),
        ("⌈ 3.2",                       &[4.0],                                 "monadic ceiling"),
        ("⌊ 3.8",                       &[3.0],                                 "monadic floor"),
        ("⍳ 5",                         &[1.0, 2.0, 3.0, 4.0, 5.0],           "monadic iota"),
        ("⍸ 2 0 1",                     &[1.0, 1.0, 3.0],                      "monadic where"),
        // Reduce
        ("+/ 1 2 3 4 5",               &[15.0],                                "reduce add"),
        ("×/ 1 2 3 4 5",               &[120.0],                               "reduce multiply"),
        ("-/ 1 2 3 4 5",               &[3.0],                                 "reduce subtract (right-fold)"),
        ("÷/ 100 5 2",                  &[40.0],                                "reduce divide (right-fold)"),
        ("⌈/ 3 6 9 1",                  &[9.0],                                 "monadic max reduce"),
        ("⌊/ 5 10 29 1",               &[1.0],                                 "monadic min reduce"),
        // Right-to-left / monadic chaining
        ("- ⍳ 5",                       &[-1.0,-2.0,-3.0,-4.0,-5.0],           "chain negate iota"),
        ("⍳ 3 + 2",                     &[1.0, 2.0, 3.0, 4.0, 5.0],           "chain iota add"),
        ("- 3 + 4",                     &[-7.0],                                "chain negate add"),
        ("+/ ⍳ 10",                     &[55.0],                                "chain reduce iota"),
        ("2 * ⍳ 5",                     &[2.0, 4.0, 8.0, 16.0, 32.0],         "chain power iota"),
        // Recursion / deep nesting
        ("- - 3",                       &[3.0],                                 "double negate"),
        ("- - - 3",                     &[-3.0],                                "triple negate"),
        ("| - 5",                       &[5.0],                                 "magnitude of negate"),
        ("- | - 5",                     &[-5.0],                                "negate magnitude negate"),
        // Right-to-left (no operator precedence)
        ("2 × 3 + 4",                   &[14.0],                                "rtl: mul before add"),
        ("2 + 3 × 4",                   &[14.0],                                "rtl: add before mul"),
        ("1 + 2 × 3 + 4",              &[15.0],                                "rtl: three ops"),
        ("10 - 3 - 2",                  &[9.0],                                 "rtl: chained sub"),
        // Parenthesized nesting
        ("(2 + 3) × (4 + 5)",           &[45.0],                                "parens: two groups"),
        ("(1 + 2) × (3 + (4 × 5))",     &[69.0],                                "parens: nested"),
        // Monadic inside dyadic
        ("2 × - 3",                     &[-6.0],                                "dyadic with monadic rhs"),
        ("1 + - 2 + 3",                 &[-4.0],                                "monadic negate chains into add"),
        // Monadic chains with vectors
        ("- - 1 2 3",                   &[1.0, 2.0, 3.0],                      "double negate vector"),
        ("| - 1 2 3",                   &[1.0, 2.0, 3.0],                      "magnitude negate vector"),
        // Complex compositions
        ("2 × ⍳ 3 + 2",                &[2.0, 4.0, 6.0, 8.0, 10.0],          "dyadic into monadic into dyadic"),
        ("+/ 2 × ⍳ 5",                  &[30.0],                                "reduce of dyadic of iota"),
        ("⍳ ⌈/ 3 1 5 2",               &[1.0, 2.0, 3.0, 4.0, 5.0],           "iota of max-reduce"),
        ("1 + +/ 1 2 3",                &[7.0],                                 "scalar plus reduce"),
        ("-/ ⍳ 6",                       &[-3.0],                                "reduce-sub of iota"),
        ("! +/ 1 2",                     &[6.0],                                 "factorial of reduce"),
        // Dyadic ⍳ (Index Of)
        ("1 2 3 4 5 ⍳ 3",              &[3.0],                                 "index of: scalar found"),
        ("1 2 3 4 5 ⍳ 6",              &[6.0],                                 "index of: scalar not found"),
        ("10 20 30 ⍳ 20 40 10",         &[2.0, 4.0, 1.0],                      "index of: vector mixed"),
        ("1 2 3 4 5 ⍳ 3 1 4 1 5",      &[3.0, 1.0, 4.0, 1.0, 5.0],           "index of: vector all found"),
        // Dyadic ⍸ (Interval Index)
        ("2 4 6 8 ⍸ 3 5 7",            &[1.0, 2.0, 3.0],                      "interval index: between"),
        ("10 20 30 40 ⍸ 15 25 35 45",   &[1.0, 2.0, 3.0, 4.0],                "interval index: between tens"),
        ("1 3 5 7 ⍸ 0 1 2 3 4 5 6 7 8", &[0.0,1.0,1.0,2.0,2.0,3.0,3.0,4.0,4.0], "interval index: full range"),
        // High minus (¯)
        ("¯3",                          &[-3.0],                                "high minus scalar"),
        ("¯3 + 5",                      &[2.0],                                 "high minus in expr"),
        ("1 ¯2 3 ¯4",                   &[1.0, -2.0, 3.0, -4.0],              "high minus in vector"),
        ("¯1 + ¯2",                     &[-3.0],                                "high minus both operands"),
        ("2 × ¯3",                      &[-6.0],                                "high minus rhs"),
        ("¯3.14",                       &[-3.14],                               "high minus float"),
        ("- ¯5",                        &[5.0],                                 "negate high minus"),
        // Comparison operators
        ("3 = 3",                       &[1.0],                                 "equal true"),
        ("3 = 4",                       &[0.0],                                 "equal false"),
        ("1 2 3 = 1 3 3",              &[1.0, 0.0, 1.0],                      "equal vector"),
        ("3 ≠ 4",                       &[1.0],                                 "not equal true"),
        ("3 ≠ 3",                       &[0.0],                                 "not equal false"),
        ("3 < 4",                       &[1.0],                                 "less than true"),
        ("4 < 3",                       &[0.0],                                 "less than false"),
        ("3 > 4",                       &[0.0],                                 "greater than false"),
        ("4 > 3",                       &[1.0],                                 "greater than true"),
        ("3 ≤ 4",                       &[1.0],                                 "leq less"),
        ("4 ≤ 4",                       &[1.0],                                 "leq equal"),
        ("5 ≤ 4",                       &[0.0],                                 "leq greater"),
        ("3 ≥ 4",                       &[0.0],                                 "geq less"),
        ("4 ≥ 4",                       &[1.0],                                 "geq equal"),
        ("5 ≥ 4",                       &[1.0],                                 "geq greater"),
        ("1 2 3 < 2 2 2",              &[1.0, 0.0, 0.0],                      "less than vector"),
        // Shape (monadic ⍴)
        ("⍴ 5",                         &[],                                    "shape of scalar"),
        ("⍴ 1 2 3",                     &[3.0],                                 "shape of vector"),
        ("⍴ ⍳ 0",                       &[0.0],                                 "shape of empty vector"),
        // Reshape (dyadic ⍴)
        ("3 ⍴ 1 2 3 4 5",              &[1.0, 2.0, 3.0],                      "reshape truncate"),
        ("5 ⍴ 1 2",                     &[1.0, 2.0, 1.0, 2.0, 1.0],           "reshape cycle"),
        ("2 3 ⍴ ⍳ 6",                   &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],     "reshape matrix"),
        ("2 3 ⍴ 1",                     &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0],     "reshape scalar cycle"),
        ("⍴ 2 3 ⍴ ⍳ 6",                &[2.0, 3.0],                           "shape of reshaped matrix"),
        // Ravel (monadic ,)
        (", 2 3 ⍴ ⍳ 6",                &[1.0,2.0,3.0,4.0,5.0,6.0],           "ravel matrix"),
        (", 5",                          &[5.0],                                "ravel scalar"),
        (", 1 2 3",                      &[1.0, 2.0, 3.0],                     "ravel vector"),
        // Catenate (dyadic ,)
        ("1 2 3 , 4 5 6",               &[1.0,2.0,3.0,4.0,5.0,6.0],          "catenate vectors"),
        ("1 , 2 3",                      &[1.0, 2.0, 3.0],                     "catenate scalar vector"),
        ("1 2 , 3",                      &[1.0, 2.0, 3.0],                     "catenate vector scalar"),
        // Reverse (monadic ⌽)
        ("⌽ 1 2 3 4 5",                 &[5.0,4.0,3.0,2.0,1.0],              "reverse vector"),
        ("⌽ 5",                          &[5.0],                                "reverse scalar"),
        // Rotate (dyadic ⌽)
        ("2 ⌽ 1 2 3 4 5",              &[3.0,4.0,5.0,1.0,2.0],              "rotate left 2"),
        ("¯1 ⌽ 1 2 3 4 5",             &[5.0,1.0,2.0,3.0,4.0],              "rotate right 1"),
        ("0 ⌽ 1 2 3 4 5",              &[1.0,2.0,3.0,4.0,5.0],              "rotate zero"),
        // Transpose (monadic ⍉)
        ("⍉ 2 3 ⍴ ⍳ 6",                &[1.0,4.0,2.0,5.0,3.0,6.0],          "transpose matrix"),
        ("⍴ ⍉ 2 3 ⍴ ⍳ 6",              &[3.0, 2.0],                          "shape of transposed"),
        ("⍉ 1 2 3",                      &[1.0, 2.0, 3.0],                     "transpose vector noop"),
        ("⍉ 5",                          &[5.0],                                "transpose scalar noop"),
        // Bool ops (∧ ∨ ⍲ ⍱)
        ("1 ∧ 1",                       &[1.0],                                 "and 1 1"),
        ("1 ∧ 0",                       &[0.0],                                 "and 1 0"),
        ("0 ∧ 0",                       &[0.0],                                 "and 0 0"),
        ("1 1 0 0 ∧ 1 0 1 0",          &[1.0, 0.0, 0.0, 0.0],                "and vector"),
        ("1 ∨ 0",                       &[1.0],                                 "or 1 0"),
        ("0 ∨ 0",                       &[0.0],                                 "or 0 0"),
        ("1 1 0 0 ∨ 1 0 1 0",          &[1.0, 1.0, 1.0, 0.0],                "or vector"),
        ("1 ⍲ 1",                       &[0.0],                                 "nand 1 1"),
        ("1 ⍲ 0",                       &[1.0],                                 "nand 1 0"),
        ("0 ⍲ 0",                       &[1.0],                                 "nand 0 0"),
        ("1 1 0 0 ⍲ 1 0 1 0",          &[0.0, 1.0, 1.0, 1.0],                "nand vector"),
        ("1 ⍱ 0",                       &[0.0],                                 "nor 1 0"),
        ("0 ⍱ 0",                       &[1.0],                                 "nor 0 0"),
        ("1 1 0 0 ⍱ 1 0 1 0",          &[0.0, 0.0, 0.0, 1.0],                "nor vector"),
        // Replicate (/ dyadic)
        ("1 0 1 0 1 / 1 2 3 4 5",      &[1.0, 3.0, 5.0],                     "replicate filter"),
        ("3 / 7",                        &[7.0, 7.0, 7.0],                     "replicate scalar"),
        ("2 1 0 / 1 2 3",              &[1.0, 1.0, 2.0],                      "replicate expand"),
        ("1 0 1 / 10 20 30",            &[10.0, 30.0],                         "replicate select"),
        // Scan (\)
        ("+\\ 1 2 3 4 5",               &[1.0, 3.0, 6.0, 10.0, 15.0],        "scan add"),
        ("×\\ 1 2 3 4 5",               &[1.0, 2.0, 6.0, 24.0, 120.0],       "scan multiply"),
        ("-\\ 1 2 3 4 5",               &[1.0, -1.0, 2.0, -2.0, 3.0],        "scan subtract"),
        ("÷\\ 100 5 2",                 &[100.0, 20.0, 40.0],                 "scan divide"),
        // Outer product (∘.)
        ("1 2 3 ∘.× 1 2 3",            &[1.0,2.0,3.0,2.0,4.0,6.0,3.0,6.0,9.0], "outer product multiply"),
        ("⍴ 1 2 3 ∘.× 1 2 3",          &[3.0, 3.0],                          "outer product shape"),
        ("1 2 3 ∘.+ 10 20",             &[11.0,21.0,12.0,22.0,13.0,23.0],    "outer product add"),
        ("⍴ 1 2 3 ∘.+ 10 20",           &[3.0, 2.0],                          "outer product add shape"),
        ("1 2 ∘.= 1 2 3",              &[1.0,0.0,0.0,0.0,1.0,0.0],           "outer product equal"),
        // Take (↑)
        ("3 ↑ 1 2 3 4 5",              &[1.0, 2.0, 3.0],                     "take first 3"),
        ("¯2 ↑ 1 2 3 4 5",             &[4.0, 5.0],                           "take last 2"),
        ("7 ↑ 1 2 3",                   &[1.0,2.0,3.0,0.0,0.0,0.0,0.0],      "take with pad"),
        ("¯5 ↑ 1 2 3",                  &[0.0,0.0,1.0,2.0,3.0],              "take with left pad"),
        // Drop (↓)
        ("2 ↓ 1 2 3 4 5",              &[3.0, 4.0, 5.0],                     "drop first 2"),
        ("¯2 ↓ 1 2 3 4 5",             &[1.0, 2.0, 3.0],                     "drop last 2"),
        ("0 ↓ 1 2 3",                   &[1.0, 2.0, 3.0],                     "drop zero"),
        // Grade up/down (⍋⍒)
        ("⍋ 3 1 4 1 5 9",              &[2.0,4.0,1.0,3.0,5.0,6.0],           "grade up"),
        ("⍒ 3 1 4 1 5 9",              &[6.0,5.0,3.0,1.0,2.0,4.0],           "grade down"),
        ("⍋ 5 4 3 2 1",                 &[5.0,4.0,3.0,2.0,1.0],              "grade up reversed"),
        // Dfns (monadic)
        ("{⍵+1} 5",                     &[6.0],                                 "dfn monadic simple"),
        ("{⍵×⍵} 1 2 3 4 5",             &[1.0,4.0,9.0,16.0,25.0],             "dfn monadic square"),
        ("{+/⍵} 1 2 3 4 5",             &[15.0],                                "dfn monadic reduce"),
        // Dfns (dyadic)
        ("2 {⍺+⍵} 3",                  &[5.0],                                 "dfn dyadic add"),
        ("10 {⍺×⍵} 1 2 3",             &[10.0, 20.0, 30.0],                   "dfn dyadic mul vector"),
        ("{⍵ + {⍵×2} 3} 10",           &[16.0],                                "nested dfn scoping"),
        ("5 {⍺ + {⍵×⍵} ⍵} 3",         &[14.0],                                "nested dfn alpha omega"),
        // Guards
        ("{⍵>0: ⍵ ⋄ 0} 5",             &[5.0],                                "guard true"),
        ("{⍵>0: ⍵ ⋄ 0} ¯3",            &[0.0],                                "guard false"),
        ("{⍵=0: 100 ⋄ ⍵=1: 200 ⋄ 300} 0", &[100.0],                          "multi guard first"),
        ("{⍵=0: 100 ⋄ ⍵=1: 200 ⋄ 300} 1", &[200.0],                          "multi guard second"),
        ("{⍵=0: 100 ⋄ ⍵=1: 200 ⋄ 300} 5", &[300.0],                          "multi guard fallback"),
        // Self-reference (∇)
        // Expand (\ dyadic)
        ("1 0 1 0 1 \\ 1 2 3",          &[1.0, 0.0, 2.0, 0.0, 3.0],          "expand with zeros"),
        ("1 1 0 1 \\ 1 2 3",            &[1.0, 2.0, 0.0, 3.0],               "expand insert zero"),
        // Circular functions (dyadic ○)
        ("1 ○ 0",                        &[0.0],                                "sin 0"),
        ("2 ○ 0",                        &[1.0],                                "cos 0"),
        ("3 ○ 1",                        &[1.5574077246549023],                 "tan 1"),
        // Expand with repeat counts
        ("2 1 2 \\ 1 2 3",              &[1.0,1.0,2.0,3.0,3.0],              "expand repeat counts"),
        // Inner product
        ("1 2 3 +.× 4 5 6",            &[32.0],                               "inner product vector"),
        ("⍴ (2 3 ⍴ ⍳ 6) +.× 3 2 ⍴ ⍳ 6", &[2.0, 2.0],                       "inner product matrix shape"),
        (", (2 3 ⍴ ⍳ 6) +.× 3 2 ⍴ ⍳ 6", &[22.0, 28.0, 49.0, 64.0],        "inner product matrix data"),
        // First (⊃)
        ("⊃ 1 2 3",                     &[1.0],                                "first of vector"),
        ("⊃ 5",                          &[5.0],                                "first of scalar"),
        // Unique (∪)
        ("∪ 1 2 3 2 1 4",              &[1.0, 2.0, 3.0, 4.0],                "unique"),
        // Union (dyadic ∪)
        ("1 2 3 ∪ 3 4 5",              &[1.0, 2.0, 3.0, 4.0, 5.0],          "union"),
        // Intersection (∩)
        ("1 2 3 ∩ 2 3 4",              &[2.0, 3.0],                           "intersection"),
        // Without (~)
        ("1 2 3 4 5 ~ 2 4",            &[1.0, 3.0, 5.0],                     "without"),
        // Not (monadic ~)
        ("~ 0 1 1 0",                   &[1.0, 0.0, 0.0, 1.0],               "not"),
        // Decode (⊥)
        ("2 ⊥ 1 0 1",                  &[5.0],                                "decode binary"),
        ("10 ⊥ 1 2 3",                  &[123.0],                              "decode decimal"),
        // Encode (⊤)
        ("2 2 2 ⊤ 5",                  &[1.0, 0.0, 1.0],                     "encode binary"),
        ("10 10 10 ⊤ 123",              &[1.0, 2.0, 3.0],                     "encode decimal"),
    ];

    let mut failures = Vec::new();
    for (expr, expected, desc) in cases {
        let result = std::panic::catch_unwind(|| assert_apl(expr, expected, desc));
        if let Err(e) = result {
            let msg = e.downcast_ref::<String>().map(|s| s.as_str())
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
    assert_apl_env("⍳ a", &mut env, &[1.0, 2.0, 3.0, 4.0, 5.0], "iota of variable");

    // Named functions
    assert_apl_env("double←{⍵×2}", &mut env, &[0.0], "assign dfn");
    assert_apl_env("double 5", &mut env, &[10.0], "call named monadic");
    assert_apl_env("double 1 2 3", &mut env, &[2.0, 4.0, 6.0], "call named monadic vector");
    assert_apl_env("add←{⍺+⍵}", &mut env, &[0.0], "assign dyadic dfn");
    assert_apl_env("10 add 20", &mut env, &[30.0], "call named dyadic");
    assert_apl_env("1 2 3 add 4 5 6", &mut env, &[5.0, 7.0, 9.0], "call named dyadic vector");
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
            assert_apl("{⍵<2: ⍵ ⋄ (∇ ⍵-1)+∇ ⍵-2} 10", &[55.0], "recursive fibonacci");
        })
        .unwrap()
        .join()
        .unwrap();
}
