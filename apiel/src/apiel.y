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
    //| Factor 'IOTA' Term {
    //    Ok(Expr::IndexOf{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
    //  }
    //| Factor 'IOTA_U' Term {
    //    Ok(Expr::IntervalIndex{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
    //  }
    | MonadicFactor {
        Ok($1?)
      }
    | Reduction { $1 } 
    | Factor { $1 } 
    ;

MonadicFactor -> Result<Expr, ()>:
      '+' Factor {
        Ok(Expr::Conjugate{ span: $span, arg: Box::new($2?) })
      }
    | '-' Factor {
        Ok(Expr::Negate{ span: $span, arg: Box::new($2?) })
      }
    | '×' Factor {
        Ok(Expr::Direction{ span: $span, arg: Box::new($2?) })
      }
    | '÷' Factor {
        Ok(Expr::Reciprocal{ span: $span, arg: Box::new($2?) })
      }
    | 'EXP' Factor {
        Ok(Expr::Exp{ span: $span, arg: Box::new($2?) })
      }
    | 'LOG' Factor {
        Ok(Expr::NaturalLog{ span: $span, arg: Box::new($2?) })
      }
    | 'CIRCLE' Factor {
        Ok(Expr::PiMultiple{ span: $span, arg: Box::new($2?) })
      }
    | '!' Factor {
        Ok(Expr::Factorial{ span: $span, arg: Box::new($2?) })
      }
    | '?' Factor {
        Ok(Expr::Roll{ span: $span, arg: Box::new($2?) })
      }
    | '|' Factor {
        Ok(Expr::Magnitude{ span: $span, arg: Box::new($2?) })
      }
    | '⌈' Factor {
        Ok(Expr::Ceil{ span: $span, arg: Box::new($2?) })
      }
    | '⌊' Factor {
        Ok(Expr::Floor{ span: $span, arg: Box::new($2?) })
      }
    | 'MAX' Factor {
        Ok(Expr::MonadicMax{ span: $span, arg: Box::new($2?) })
      }
    | 'MIN' Factor {
        Ok(Expr::MonadicMin{ span: $span, arg: Box::new($2?) })
      }
    | 'IOTA' Factor {
        Ok(Expr::GenIndex{ span: $span, arg: Box::new($2?) })
      }
    | 'IOTA_U' Factor {
        Ok(Expr::Where{ span: $span, arg: Box::new($2?) })
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
                
                #[cfg(feature = "debug")]
                {
                    !("Trying to parse vec: {}", full_str);
                }

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
    ;

    Reduction -> Result<Expr, ()>:
    Operator '/' Term {
        match $1 {
            Ok(op) => Ok(Expr::Reduce{ span: $span, operator: op, term: Box::new($3?) }),
            Err(_) => Err(())
        }
    }
    ;

    Operator -> Result<Operator, ()>:
      '+' { Ok(Operator::Add) }
    | '-' { Ok(Operator::Subtract) }
    | '×' { Ok(Operator::Multiply) }
    | '÷' { Ok(Operator::Divide) }
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
    //IndexOf {
    //    span: Span,
    //    lhs: Box<Expr>,
    //    rhs: Box<Expr>,
    //},
    //IntervalIndex {
    //    span: Span,
    //    lhs: Box<Expr>,
    //    rhs: Box<Expr>,
    //},

    // Monadic

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
}