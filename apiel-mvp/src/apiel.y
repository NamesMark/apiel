%start Expr
%avoid_insert "INT"
%expect-unused Unmatched "UNMATCHED"
%%
Expr -> Result<Expr, ()>:
    Term { $1 }
    ;

Term -> Result<Expr, ()>:
      Factor '+' Term  {
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
    | MonadicFactor {
        Ok($1?)
      }
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
    | 'MAX' Factor {
        Ok(Expr::Max{ span: $span, arg: Box::new($2?) })
      }
    |'MIN' Factor {
        Ok(Expr::Min{ span: $span, arg: Box::new($2?) })
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
                    println!("Trying to parse vec: {}", full_str);
                }

                for value in full_str.split_whitespace() {
                    let start = full_str[current_pos..].find(value).unwrap_or(0) + current_pos;
                    let end = start + value.len();
                    current_pos = end; 

                    elements.push(Expr::Scalar { span: Span::new(start + $span.start(), end + $span.start()) });
                }
                elements
            },
            Err(_) => Vec::new(), 
        };
        Ok(Expr::Vector { span: $span, elements })
    }
    | 'INT' {
        match $1 {
            Ok(_) => Ok(Expr::Scalar { span: $span }),
            Err(_) => Err(())
        }
    }
    ;

Unmatched -> ():
      "UNMATCHED" { }
    ;
%%

use cfgrammar::Span;

#[derive(Debug)]
pub enum Expr {
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
    Exp {
        span: Span,
        arg: Box<Expr>,
    },
    Log {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
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
    Ceil {
        span: Span,
        arg: Box<Expr>,
    },
    Floor {
        span: Span,
        arg: Box<Expr>,
    },
    Max {
        span: Span,
        arg: Box<Expr>,
    },
    Min {
        span: Span,
        arg: Box<Expr>,
    },
    Scalar {
        span: Span,
    },
    Vector {
        span: Span,
        elements: Vec<Expr>,
    },
}