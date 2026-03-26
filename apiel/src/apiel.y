%start Expr
%avoid_insert "INT"
%expect-unused Unmatched "UNMATCHED"
%%
Expr -> Result<Expr, ()>:
    Term { $1 }
    ;

Term -> Result<Expr, ()>:
      Factor '+' Term {
        Ok(Expr::Add{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '-' Term {
        Ok(Expr::Sub{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '×' Term {
        Ok(Expr::Mul{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '÷' Term {
        Ok(Expr::Div{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'EXP' Term {
        Ok(Expr::Power{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '!' Term {
        Ok(Expr::Binomial{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '?' Term {
        Ok(Expr::Deal{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '|' Term {
        Ok(Expr::Residue{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '⌈' Term {
        Ok(Expr::Max{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '⌊' Term {
        Ok(Expr::Min{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'LOG' Term {
        Ok(Expr::Log{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'IOTA' Term {
        Ok(Expr::IndexOf{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'IOTA_U' Term {
        Ok(Expr::IntervalIndex{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'EQ' Term {
        Ok(Expr::Equal{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'NEQ' Term {
        Ok(Expr::NotEqual{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'LT' Term {
        Ok(Expr::LessThan{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'GT' Term {
        Ok(Expr::GreaterThan{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'LTE' Term {
        Ok(Expr::LessEqual{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'GTE' Term {
        Ok(Expr::GreaterEqual{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'RHO' Term {
        Ok(Expr::Reshape{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor ',' Term {
        Ok(Expr::Catenate{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'ROTATE' Term {
        Ok(Expr::Rotate{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'CIRCLE' Term {
        Ok(Expr::Circular{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'AND' Term {
        Ok(Expr::And{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'OR' Term {
        Ok(Expr::Or{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'NAND' Term {
        Ok(Expr::Nand{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'NOR' Term {
        Ok(Expr::Nor{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '/' Term {
        Ok(Expr::Replicate{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '\' Term {
        Ok(Expr::Expand{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'TAKE' Term {
        Ok(Expr::Take{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'DROP' Term {
        Ok(Expr::Drop{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | 'NAME' 'ASSIGN' Term {
        Ok(Expr::Assign{ span: $span, name: $1.map(|l| $lexer.span_str(l.span()).to_string()).unwrap_or_default(), rhs: Box::new($3?) })
      }
    | Factor 'OUTERPRODUCT' Operator Term {
        match $3 {
            Ok(op) => Ok(Expr::OuterProduct{ span: $span, lhs: Box::new($1?), operator: op, rhs: Box::new($4?) }),
            Err(_) => Err(())
        }
      }
    | Factor '{' Expr '}' Term {
        Ok(Expr::DyadicDfn{ span: $span, lhs: Box::new($1?), body: Box::new($3?), rhs: Box::new($5?) })
      }
    | '{' Expr '}' Term {
        Ok(Expr::MonadicDfn{ span: $span, body: Box::new($2?), rhs: Box::new($4?) })
      }
    | MonadicFactor {
        Ok($1?)
      }
    | Reduction { $1 }
    | Factor { $1 }
    ;

MonadicFactor -> Result<Expr, ()>:
      '+' Term {
        Ok(Expr::Conjugate{ span: $span, arg: Box::new($2?) })
      }
    | '-' Term {
        Ok(Expr::Negate{ span: $span, arg: Box::new($2?) })
      }
    | '×' Term {
        Ok(Expr::Direction{ span: $span, arg: Box::new($2?) })
      }
    | '÷' Term {
        Ok(Expr::Reciprocal{ span: $span, arg: Box::new($2?) })
      }
    | 'EXP' Term {
        Ok(Expr::Exp{ span: $span, arg: Box::new($2?) })
      }
    | 'LOG' Term {
        Ok(Expr::NaturalLog{ span: $span, arg: Box::new($2?) })
      }
    | 'CIRCLE' Term {
        Ok(Expr::PiMultiple{ span: $span, arg: Box::new($2?) })
      }
    | '!' Term {
        Ok(Expr::Factorial{ span: $span, arg: Box::new($2?) })
      }
    | '?' Term {
        Ok(Expr::Roll{ span: $span, arg: Box::new($2?) })
      }
    | '|' Term {
        Ok(Expr::Magnitude{ span: $span, arg: Box::new($2?) })
      }
    | '⌈' Term {
        Ok(Expr::Ceil{ span: $span, arg: Box::new($2?) })
      }
    | '⌊' Term {
        Ok(Expr::Floor{ span: $span, arg: Box::new($2?) })
      }
    | 'MAX' Term {
        Ok(Expr::MonadicMax{ span: $span, arg: Box::new($2?) })
      }
    | 'MIN' Term {
        Ok(Expr::MonadicMin{ span: $span, arg: Box::new($2?) })
      }
    | 'IOTA' Term {
        Ok(Expr::GenIndex{ span: $span, arg: Box::new($2?) })
      }
    | 'IOTA_U' Term {
        Ok(Expr::Where{ span: $span, arg: Box::new($2?) })
      }
    | 'RHO' Term {
        Ok(Expr::Shape{ span: $span, arg: Box::new($2?) })
      }
    | ',' Term {
        Ok(Expr::Ravel{ span: $span, arg: Box::new($2?) })
      }
    | 'ROTATE' Term {
        Ok(Expr::Reverse{ span: $span, arg: Box::new($2?) })
      }
    | 'TRANSPOSE' Term {
        Ok(Expr::Transpose{ span: $span, arg: Box::new($2?) })
      }
    | 'GRADEUP' Term {
        Ok(Expr::GradeUp{ span: $span, arg: Box::new($2?) })
      }
    | 'GRADEDN' Term {
        Ok(Expr::GradeDown{ span: $span, arg: Box::new($2?) })
      }
    ;

Factor -> Result<Expr, ()>:
      '(' Expr ')' { $2 }

    | 'VEC' {
        let elements = match $1 {
            Ok(_lexeme) => {
                let full_str = $lexer.span_str($span);
                let mut current_pos = 0;
                let mut elements = Vec::new();

                for value in full_str.split_whitespace() {
                    let start = full_str[current_pos..].find(value).unwrap_or(0) + current_pos;
                    let end = start + value.len();
                    current_pos = end;

                    elements.push(Expr::ScalarInteger { span: Span::new(start + $span.start(), end + $span.start()) });
                }
                elements
            },
            Err(_) => Vec::new(),
        };
        Ok(Expr::Vector { span: $span, elements })
    }
    | 'INT' {
        match $1 {
            Ok(_) => Ok(Expr::ScalarInteger { span: $span }),
            Err(_) => Err(())
        }
    }
    | 'FLOAT' {
        match $1 {
            Ok(_value) => Ok(Expr::ScalarFloat { span: $span }),
            Err(_) => Err(())
        }
    }
    | 'NAME' {
        match $1 {
            Ok(_) => Ok(Expr::Variable { span: $span, name: $lexer.span_str($span).to_string() }),
            Err(_) => Err(())
        }
    }
    | 'OMEGA' {
        Ok(Expr::Omega { span: $span })
    }
    | 'ALPHA' {
        Ok(Expr::Alpha { span: $span })
    }
    ;

    Reduction -> Result<Expr, ()>:
    Operator '/' Term {
        match $1 {
            Ok(op) => Ok(Expr::Reduce{ span: $span, operator: op, term: Box::new($3?) }),
            Err(_) => Err(())
        }
    }
    | Operator '\' Term {
        match $1 {
            Ok(op) => Ok(Expr::Scan{ span: $span, operator: op, term: Box::new($3?) }),
            Err(_) => Err(())
        }
    }
    ;

    Operator -> Result<Operator, ()>:
      '+' { Ok(Operator::Add) }
    | '-' { Ok(Operator::Subtract) }
    | '×' { Ok(Operator::Multiply) }
    | '÷' { Ok(Operator::Divide) }
    | 'EQ' { Ok(Operator::Equal) }
    | 'LT' { Ok(Operator::LessThan) }
    | 'GT' { Ok(Operator::GreaterThan) }
    | '⌈' { Ok(Operator::Max) }
    | '⌊' { Ok(Operator::Min) }
    ;


Unmatched -> ():
      "UNMATCHED" { }
    ;
%%

use cfgrammar::Span;

#[derive(Debug)]
pub enum Expr {
    // Dyadic
    Add {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Sub {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Mul {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Div {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Power {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Log {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Min {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Max {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Binomial {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Deal {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Residue {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    IndexOf {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    IntervalIndex {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Equal {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    NotEqual {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    LessThan {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    GreaterThan {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    LessEqual {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    GreaterEqual {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Reshape {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Catenate {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Rotate {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    And {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Or {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Nand {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Nor {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Replicate {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Expand {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Circular {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Take {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Drop {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Assign {
        span: Span,
        name: String,
        rhs: Box<Expr>,
    },
    MonadicDfn {
        span: Span,
        body: Box<Expr>,
        rhs: Box<Expr>,
    },
    DyadicDfn {
        span: Span,
        lhs: Box<Expr>,
        body: Box<Expr>,
        rhs: Box<Expr>,
    },
    Variable {
        span: Span,
        name: String,
    },
    Omega {
        span: Span,
    },
    Alpha {
        span: Span,
    },
    OuterProduct {
        span: Span,
        lhs: Box<Expr>,
        operator: Operator,
        rhs: Box<Expr>,
    },
    Scan {
        span: Span,
        operator: Operator,
        term: Box<Expr>,
    },

    // Monadic

    Shape {
        span: Span,
        arg: Box<Expr>,
    },
    Ravel {
        span: Span,
        arg: Box<Expr>,
    },
    Reverse {
        span: Span,
        arg: Box<Expr>,
    },
    Transpose {
        span: Span,
        arg: Box<Expr>,
    },
    GradeUp {
        span: Span,
        arg: Box<Expr>,
    },
    GradeDown {
        span: Span,
        arg: Box<Expr>,
    },
    Exp {
        span: Span,
        arg: Box<Expr>,
    },
    NaturalLog {
        span: Span,
        arg: Box<Expr>,
    },
    Conjugate {
        span: Span,
        arg: Box<Expr>,
    },
    Negate {
        span: Span,
        arg: Box<Expr>,
    },
    Direction {
        span: Span,
        arg: Box<Expr>,
    },
    Reciprocal {
        span: Span,
        arg: Box<Expr>,
    },
    PiMultiple {
        span: Span,
        arg: Box<Expr>,
    },
    Factorial {
        span: Span,
        arg: Box<Expr>,
    },
    Roll {
        span: Span,
        arg: Box<Expr>,
    },
    Magnitude {
        span: Span,
        arg: Box<Expr>,
    },
    Ceil {
        span: Span,
        arg: Box<Expr>,
    },
    Floor {
        span: Span,
        arg: Box<Expr>,
    },
    MonadicMax {
        span: Span,
        arg: Box<Expr>,
    },
    MonadicMin {
        span: Span,
        arg: Box<Expr>,
    },
    GenIndex {
        span: Span,
        arg: Box<Expr>,
    },
    Where {
        span: Span,
        arg: Box<Expr>,
    },

    Reduce {
        span: Span,
        operator: Operator,
        term: Box<Expr>,
    },

    // Values

    ScalarInteger {
        span: Span,
    },
    ScalarFloat {
        span: Span,
    },
    Vector {
        span: Span,
        elements: Vec<Expr>,
    },
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    LessThan,
    GreaterThan,
    Max,
    Min,
}
