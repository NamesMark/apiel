//! Integration tests verified against Dyalog APL 19.0.

use apiel::apl;

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
