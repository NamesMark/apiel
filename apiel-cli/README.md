# apiel-cli

Interactive REPL for [apiel](https://crates.io/crates/apiel), a subset of the APL programming language implemented in Rust.

## Install

```
cargo install apiel-cli
```

## Usage

```
$ apiel-cli
>>> ⍳ 5
1 2 3 4 5
>>> +/ ⍳ 10
55
>>> 2 3 ⍴ ⍳ 6
1 2 3 4 5 6
>>> ⍴ 2 3 ⍴ ⍳ 6
2 3
>>> ⌽ 'hello'
olleh
```

Variables and functions persist across lines:

```
>>> data←⍳ 10
>>> +/ data
55
>>> double←{⍵×2}
>>> double 1 2 3
2 4 6
>>> {⍵≤1: ⍵ ⋄ ⍵×∇ ⍵-1} 5
120
```

See the [apiel](https://crates.io/crates/apiel) crate for the full support info.

## Affiliation

Capstone project for the [rustcamp](https://github.com/rust-lang-ua/rustcamp) by the [Ukrainian Rust Community](https://www.uarust.com).
