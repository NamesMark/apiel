# apiel
Apiel is a small subset of the [APL programming language](https://en.wikipedia.org/wiki/APL_(programming_language)) implemented in Rust. 

The ultimate goal of the project is to export a macro that allows evaluating APL expressions from Rust code, providing a way to solve some problems in a very conscise manner.

## Affiliation

This is my capstone project for the [rustcamp](https://github.com/rust-lang-ua/rustcamp), a Rust bootcamp organized by the Ukrainian Rust Community ([website](https://www.uarust.com), [linked in](https://www.linkedin.com/company/ukrainian-rust-community), [telegram](https://t.me/rustlang_ua), [github](https://github.com/rust-lang-ua), [youtube](https://www.youtube.com/channel/UCmkAFUu2MVOX8ly0LjB6TMA), [twitter](https://twitter.com/rustukraine)).

## Array languages

APL was the first language in an "Array programming" or "Iversonian" paradigm. These languages are closer to mathematical notation than to C-like programming languages. The concepts proposed by APL inspired many similar languages, influenced the development of the functional programming paradigm, and had a giant impact on programming as a whole.

## Approach

The project utilizes [Yacc](https://en.wikipedia.org/wiki/Yacc) **(Yet-Another-Compiler-Compiler)** implementation in Rust through [grmtools](https://github.com/softdevteam/grmtools) to build the lexer and parser. 

`apiel.l` contains the tokens for the **lexer**, `apiel.y` describes the **Yacc grammar**. The build.rs generaters Rust code for the lexer and parser generator. 

My main entry point is apiel/src/parse/mod.rs. There is `fn parse_and_evaluate()` that runs `parse::eval()` (located in /parse/eval.rs) on the expression passed to it. The `parse::eval()` contains a single match expression that performs operations on the data contained in the `Expr` enumeration according to the expression type (it always calls parse::eval() recursively for `lhs` and `rhs` of the expression). The `Expr` enumeration is defined in `aliel.y`. Ech variant of the Expr usually contains a `Span` identifying where it's located in the original input, and boxed arguments, which allows for unlimited recursion inside the expression.

## Usage

