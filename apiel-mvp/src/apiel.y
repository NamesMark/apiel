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
    | Factor { $1 }
    ;

Factor -> Result<Expr, ()>:
      '(' Expr ')' { $2 }
    | 'INT' { Ok(Expr::Number{ span: $span }) }
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
    Number {
        span: Span
    }
}