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
fn dyalog_reference_battery() {
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
