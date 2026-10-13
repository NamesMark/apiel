#![deny(rust_2018_idioms)]
use cfgrammar::yacc::YaccKind;
use lrlex::CTLexerBuilder;

fn main() {
    // Since we're using both lrlex and lrpar, we use lrlex's `lrpar_config` convenience function
    // that makes it easy to a) create a lexer and parser and b) link them together.
    CTLexerBuilder::new()
        .rust_edition(lrlex::RustEdition::Rust2021)
        .lrpar_config(|ctp| {
            // Error recovery times itself with std::time::Instant, which panics on
            // wasm32-unknown-unknown.
            ctp.yacckind(YaccKind::Grmtools)
                .recoverer(lrpar::RecoveryKind::None)
                .rust_edition(lrpar::RustEdition::Rust2021)
                .error_on_conflicts(false)
                .grammar_in_src_dir("apiel.y")
                .unwrap()
        })
        .lexer_in_src_dir("apiel.l")
        .unwrap()
        .build()
        .unwrap();
}
