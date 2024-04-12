use std::str::FromStr;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use eyre::WrapErr as _;

#[derive(Debug, PartialEq)]
pub enum Operation {
    // Unary
    Negate,

    Max,
    Min,

    // Binary
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Num(i32),
    UnaryOp {
        operation: Operation,
        operand: Box<Expr>,
    },
    BinaryOp {
        operation: Operation,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

fn parse(expr: &str) -> eyre::Result<String> {
    unimplemented!()
}

fn evaluate(expr: &str) -> eyre::Result<String> {
    unimplemented!()
}

fn evaluate_as<T>(expr: &str) -> eyre::Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    let result = evaluate(expr)?;
    match result.parse::<T>() {
        Ok(output) => Ok(output),
        Err(e) => {
            warn!(
                "Couldn't parse {result} into {}",
                std::any::type_name::<T>()
            );
            eyre::bail!(
                "Couldn't parse {result} into {}: {:?}",
                std::any::type_name::<T>(),
                e
            )
        }
    }
}

fn parse_vec<T: FromStr>(input: &str) -> eyre::Result<Vec<T>>
where
    T::Err: std::fmt::Display + std::error::Error + Send + Sync + 'static,
{
    input.split_whitespace()
         .map(|s| s.parse::<T>().wrap_err("Failed to parse")) 
         .collect() 
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_sum() {
        assert_eq!(evaluate("2+2").expect("Couldn't evaluate expression"), "4");
        assert_eq!(
            evaluate_as::<i32>("2+2").expect("Couldn't evaluate or convert to a number"),
            4
        );
    }

    #[test]
    fn array_sum() {
        assert_eq!(evaluate("4 2 3 + 8 5 7").expect("12 7 10"), "4");
        assert_eq!(
            evaluate_as::<Vec<i32>>("4 2 3 + 8 5 7").expect("Couldn't evaluate or convert to a number"),
            [12, 7, 10].to_vec()
        );
    }

    const arithmetic_tests: [(&str, &str); 7] = [
        ("+/⍳10", "55"),      // Summation
        ("×/⍳5", "120"),      // Product
        ("-/1 2 3 4", "-8"),  // Reduction
        ("⌈/1 5 2 9 3", "9"), // Maximum
        ("⌊/1 5 2 9 3", "1"), // Minimum
        ("÷/1 2 4", "0.125"), // Division
        ("(+/⍳10)÷10", "5.5"), // Average
                              // ("⍳10", "1 2 3 4 5 6 7 8 9 10"),       // Generate Sequence
                              // ("3 3⍴⍳9", "1 2 3 4 5 6 7 8 9"),       // Reshape
                              // ("(3⌷⍳10)", "3"),                      // Indexing
    ];

    #[test]
    fn arithmetic_works() {
        for (input, expected) in arithmetic_tests.iter() {
            let result = evaluate(input).expect("Couldn't evaluate expression");
            assert_eq!(&result, expected, "Test failed for input: {}", input);
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(2, 2);
    }
}
