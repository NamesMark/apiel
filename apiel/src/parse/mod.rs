pub mod eval;
pub mod val;

use cfgrammar::Span;
use lrlex::{DefaultLexerTypes, lrlex_mod};
use lrpar::{Lexeme, Lexer, NonStreamingLexer, lrpar_mod};

lrlex_mod!("apiel.l");
lrpar_mod!("apiel.y");

pub use eval::Env;
use val::{Scalar, Val};

pub fn parse_and_evaluate(line: &str) -> Result<Vec<f64>, String> {
    let mut env = Env::new();
    parse_and_evaluate_with_env(line, &mut env)
}

pub fn parse_and_evaluate_with_env(line: &str, env: &mut Env) -> Result<Vec<f64>, String> {
    eval_to_val(line, env).map(|val| val.data.into_iter().map(f64::from).collect())
}

// --- Token-level train rewriting ---

/// A token with its byte span and text, extracted from the lexer.
struct Tok<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

/// Is this token text a primitive dyadic operator (usable in trains)?
fn is_operator_tok(t: &str) -> bool {
    matches!(
        t,
        "+" | "-" | "×" | "÷" | "*" | "⍟" | "○" | "!" | "?" | "|" | "⌈" | "⌊"
            | "=" | "≠" | "<" | ">" | "≤" | "≥" | "∧" | "∨" | "⍲" | "⍱" | ","
    )
}

/// Is this token text a monadic-only function (usable in trains)?
fn is_monadic_fn_tok(t: &str) -> bool {
    matches!(
        t,
        "⍴" | "⌽" | "⍳" | "⍋" | "⍒" | "≢" | "≡" | "∪" | "⊃" | "⊂" | "⍉" | "~" | "⊣"
            | "⊢" | "⌹" | "⍸" | "⍷" | "↑" | "↓"
    )
}

/// Is this token text a pre-composed reduction (single lexer token)?
fn is_builtin_reduce_tok(t: &str) -> bool {
    matches!(t, "⌈/" | "⌊/")
}

/// Is this token text a NAME (identifier)?
fn is_name_tok(t: &str) -> bool {
    let mut chars = t.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

/// Is this token text a value (NOT function-like)?
fn is_value_tok(t: &str) -> bool {
    // INT, FLOAT, VEC, STRING, OMEGA, ALPHA, braces, brackets, assignment, etc.
    if t == "⍵" || t == "⍺" || t == "←" || t == "⋄" || t == ":" {
        return true;
    }
    if t.starts_with('\'') {
        return true; // STRING
    }
    if t == "{" || t == "}" || t == "[" || t == "]" {
        return true;
    }
    // Numeric: starts with digit or ¯ followed by digit
    let first = t.chars().next().unwrap_or(' ');
    if first.is_ascii_digit() {
        return true;
    }
    if first == '¯' && t.len() > 1 {
        return true;
    }
    false
}

/// A function reference parsed from the token stream.
enum TrainFn {
    /// A primitive operator or monadic function: "+", "≢", "⍴", etc.
    Simple(String),
    /// A derived function (reduce/scan): "+/", "×\", "⌈/", etc.
    Derived(String),
    /// A named user-defined function
    Named(String),
}

impl TrainFn {
    /// Build monadic application string: `f⍵`
    fn apply_monadic(&self) -> String {
        match self {
            TrainFn::Simple(f) | TrainFn::Derived(f) => format!("{f}⍵"),
            TrainFn::Named(f) => format!("{f} ⍵"),
        }
    }

    /// Return the text of this function reference (for dyadic use between results).
    fn text(&self) -> &str {
        match self {
            TrainFn::Simple(f) | TrainFn::Derived(f) | TrainFn::Named(f) => f,
        }
    }
}

/// Try to parse a sequence of tokens (between parens) as train function references.
/// Returns None if any token is value-like or the count isn't 2 or 3.
fn try_parse_train(tokens: &[Tok]) -> Option<Vec<TrainFn>> {
    if tokens.is_empty() {
        return None;
    }

    // Reject if any value-like token is present
    if tokens.iter().any(|t| is_value_tok(t.text)) {
        return None;
    }

    // Reject if nested parens/braces are present
    if tokens.iter().any(|t| matches!(t.text, "(" | ")" | "{" | "}")) {
        return None;
    }

    let mut fns: Vec<TrainFn> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let t = tokens[i].text;

        // Built-in reductions that are single tokens (⌈/ ⌊/)
        if is_builtin_reduce_tok(t) {
            fns.push(TrainFn::Derived(t.to_string()));
            i += 1;
            continue;
        }

        // Operator possibly followed by / or \ (reduce/scan)
        if is_operator_tok(t) {
            if i + 1 < tokens.len() && (tokens[i + 1].text == "/" || tokens[i + 1].text == "\\") {
                fns.push(TrainFn::Derived(format!("{}{}", t, tokens[i + 1].text)));
                i += 2;
                continue;
            }
            fns.push(TrainFn::Simple(t.to_string()));
            i += 1;
            continue;
        }

        // Monadic-only function
        if is_monadic_fn_tok(t) {
            fns.push(TrainFn::Simple(t.to_string()));
            i += 1;
            continue;
        }

        // NAME (user-defined function)
        if is_name_tok(t) {
            fns.push(TrainFn::Named(t.to_string()));
            i += 1;
            continue;
        }

        // Unrecognized token -> not a train
        return None;
    }

    if fns.len() == 2 || fns.len() == 3 {
        Some(fns)
    } else {
        None
    }
}

/// Build a dfn string from train function references.
fn build_train_dfn(fns: &[TrainFn]) -> String {
    if fns.len() == 3 {
        // Fork: (f g h) -> {(f⍵)g(h⍵)}
        let f_app = fns[0].apply_monadic();
        let g = fns[1].text();
        let h_app = fns[2].apply_monadic();
        format!("{{({f_app}){g}({h_app})}}")
    } else {
        // Atop: (f g) -> {f(g⍵)}
        let f = fns[0].text();
        let g_app = fns[1].apply_monadic();
        format!("{{{f}({g_app})}}")
    }
}

/// Rewrite train patterns using token-level analysis.
///
/// Tokenizes the input with the lexer, identifies parenthesized groups containing
/// only function-like tokens (operators, monadic functions, derived functions, names),
/// and rewrites them as dfn expressions.
fn rewrite_trains(input: &str) -> String {
    let lexerdef = apiel_l::lexerdef();
    let lexer = lexerdef.lexer(input);

    // Collect tokens with byte spans
    let tokens: Vec<Tok> = lexer
        .iter()
        .filter_map(|r| r.ok())
        .map(|tok| {
            let s = tok.span();
            Tok {
                start: s.start(),
                end: s.end(),
                text: &input[s.start()..s.end()],
            }
        })
        .collect();

    // Find parenthesized groups and check for trains
    // Collect (paren_open_byte, paren_close_byte_end, replacement_string)
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].text == "(" {
            // Find matching ) at depth 0
            let mut depth = 1;
            let mut j = i + 1;
            while j < tokens.len() && depth > 0 {
                match tokens[j].text {
                    "(" => depth += 1,
                    ")" => depth -= 1,
                    _ => {}
                }
                if depth > 0 {
                    j += 1;
                }
            }
            if depth == 0 && j > i + 1 {
                // Inner tokens: i+1 .. j (exclusive of parens)
                let inner = &tokens[i + 1..j];
                if let Some(fns) = try_parse_train(inner) {
                    let replacement = build_train_dfn(&fns);
                    replacements.push((tokens[i].start, tokens[j].end, replacement));
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    if replacements.is_empty() {
        return input.to_string();
    }

    // Apply replacements in reverse order so byte offsets stay valid
    let mut result = input.to_string();
    for (start, end, replacement) in replacements.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }
    result
}

pub fn eval_to_val(line: &str, env: &mut Env) -> Result<Val, String> {
    let line = &rewrite_trains(line);
    let lexerdef = apiel_l::lexerdef();
    let lexer = lexerdef.lexer(line);

    {
        let mut tokens = String::new();
        for token in lexer.iter() {
            match token {
                Ok(token) => tokens.push_str(&format!("{} ", token.tok_id())),
                Err(e) => {
                    tracing::warn!("Failed to parse a token: {e}");
                    tokens.push_str("UNKNOWN");
                }
            }
        }
        tracing::debug!(tokens, "Tokens:");
    }

    let (res, errs) = apiel_y::parse(&lexer);

    if !errs.is_empty() {
        return Err(format!("Parse error: {:?}", errs));
    }

    if let Some(Ok(r)) = res {
        eval::eval(&lexer, r, env).map_err(|(span, msg)| {
            let ((line, col), _) = lexer.line_col(span);
            format!(
                "Evaluation error at line {} column {}: '{}', {}.",
                line,
                col,
                lexer.span_str(span),
                msg
            )
        })
    } else {
        Err("Failed to evaluate expression".to_string())
    }
}

pub fn format_val(val: &Val) -> String {
    if val.data.iter().all(|s| matches!(s, Scalar::Char(_))) {
        // All chars: display as string
        val.data
            .iter()
            .map(|s| match s {
                Scalar::Char(c) => *c,
                _ => ' ',
            })
            .collect()
    } else {
        val.data
            .iter()
            .map(|v| match v {
                Scalar::Integer(i) => format!("{i}"),
                Scalar::Float(f) if f.fract() == 0.0 => format!("{}", *f as i64),
                Scalar::Float(f) => format!("{f}"),
                Scalar::Char(c) => format!("{c}"),
                Scalar::Nested(v) => format!("({})", format_val(v)),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
