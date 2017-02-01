relint
------------
`relint` is a line oriented lint tool with support for fixing issues.

[![Linux build status](https://travis-ci.org/epage/relint.svg?branch=master)](https://travis-ci.org/epage/relint) [![Windows build status](https://ci.appveyor.com/api/projects/status/e0uku9mx912s8962/branch/master?svg=true)](https://ci.appveyor.com/project/epage/relint/branch/master)[![](https://tokei.rs/b1/github/epage/relint)](https://github.com/epage/relint)

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org).

### Installation

If you're a **Rust programmer**, `relint` can be installed with `cargo`:

```
$ cargo install relint
```

`relint` isn't currently in any other package repositories.
[I'd like to change that](https://github.com/epage/relint/issues/1).

### Whirlwind tour

### Regex syntax

The syntax supported is
[documented as part of Rust's regex library](https://doc.rust-lang.org/regex/regex/index.html#syntax).

### Building

`relint` is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
`relint` compiles with Rust 1.13 (stable) or newer. Building is easy:

```
$ git clone https://github.com/epage/relint
$ cd relint
$ cargo build --release
$ ./target/release/relint --version
0.1.0
```

### Running tests

`relint` tries to be well tested, including both unit tests and integration
tests. To run the full test suite, use:

```
$ cargo test
```

from the repository root.
