%start Expr
%avoid_insert "INT"
%expect-unused Unmatched "UNMATCHED"
%%
Expr -> Result<Expr, ()>:
      Term '+' Expr  {
        Ok(Expr::Add{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | 
      Term '-' Expr {
        Ok(Expr::Sub{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Term { $1 }
    ;

Term -> Result<Expr, ()>:
      Factor '×' Term {
        Ok(Expr::Mul{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor '÷' Term {
        Ok(Expr::Div{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | Factor 'EXP' Term {
        Ok(Expr::Exp{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | MonadicFactor { $1 }
    ;

MonadicFactor -> Result<Expr, ()>:
      'MAX' Factor {
        Ok(Expr::Max{ span: $span, arg: Box::new($2?) })
      }
    | 'MIN' Factor {
        Ok(Expr::Min{ span: $span, arg: Box::new($2?) })
      }
    | Factor { $1 }
    ;

Factor -> Result<Expr, ()>:
      '(' Expr ')' { $2 }

    | 'VEC' {
        let elements = match $1 {
            Ok(lexeme) => {
                let full_span = lexeme.span();
                let full_str = $lexer.span_str(full_span);
                let mut current_pos = full_span.start();
                println!("Trying to parse vec: {}", full_str);
                full_str.split_whitespace().map(|value| {
                    let start = full_str[current_pos..].find(value).unwrap_or(0) + current_pos;
                    let end = start + value.len();
                    current_pos = end;
                    Expr::Scalar { span: Span::new(start, end) }
                }).collect::<Vec<_>>()
            },
            Err(_) => {
                vec![]
            }
        };
        Ok(Expr::Vector { span: $span, elements })
    }
    | 'INT' {
        match $1 {
            Ok(lexeme) => Ok(Expr::Scalar { span: $span }),
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
    Exp {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
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