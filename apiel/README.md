# apiel
This is the library crate for the **apiel** interpreter.

**apiel** is a small subset of the [APL programming language](https://en.wikipedia.org/wiki/APL_(programming_language)) implemented in Rust. 

The ultimate goal of the project is to export a macro that allows evaluating APL expressions from Rust code, providing a way to solve some problems in a very conscise manner.

## Affiliation

This was created as a capstone project for the [rustcamp](https://github.com/rust-lang-ua/rustcamp), a Rust bootcamp organized by the Ukrainian Rust Community ([website](https://www.uarust.com), [linked in](https://www.linkedin.com/company/ukrainian-rust-community), [telegram](https://t.me/rustlang_ua), [github](https://github.com/rust-lang-ua), [youtube](https://www.youtube.com/channel/UCmkAFUu2MVOX8ly0LjB6TMA), [twitter](https://twitter.com/rustukraine)).

## Usage 

Add the crate to the project.

Call the `parse_and_evaluate()` function:

```rust
match apiel::parse::parse_and_evaluate(&line) {
    Ok(result) => println!("Result: {:?}", result),
    Err(err) => (), // process the error
}
```