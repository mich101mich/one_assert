A Rust crate with a more powerful `assert!()` macro

[![Tests](https://github.com/mich101mich/one_assert/actions/workflows/test.yml/badge.svg)](https://github.com/mich101mich/one_assert/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/one_assert.svg)](https://crates.io/crates/one_assert)
[![Documentation](https://docs.rs/one_assert/badge.svg)](https://docs.rs/one_assert/)
[![Dependency status](https://deps.rs/repo/github/mich101mich/one_assert/status.svg)](https://deps.rs/repo/github/mich101mich/one_assert)

# One Assert

### TL;DR
Why have separate macros for `assert_eq` and `assert_ne` (and `assert_gt` etc. with other crates) when you can
just get the same output with `assert!(a == b)` (or `assert!(a != b)`, `assert!(a > b)`, …)? This crate provides a
single `assert!` macro that analyzes the expression to provide more detailed output on failure.

### Introduction
Rust's standard library provides the [`assert`], [`assert_eq`] and [`assert_ne`] macros. There are however some
inconveniences with these, like how there are no specialization for other inequalities, like `assert_ge` for `>=`
etc, or how the names only differ in one or two letters (`assert_eq`, `assert_ne`, `assert_ge`, `assert_gt`, …)
and are thus easy to mix up at a glance.

[`assert`]:    https://doc.rust-lang.org/std/macro.assert.html
[`assert_eq`]: https://doc.rust-lang.org/std/macro.assert_eq.html
[`assert_ne`]: https://doc.rust-lang.org/std/macro.assert_ne.html

The main reason for not adding more macros is that they can be represented just fine with `assert!(a >= b)`, so
there is no need for a separate macro for every use case.

But that begs the question: Why do we have `assert_eq` and `assert_ne` in the first place?

The practical reason: `assert_eq!(a, b)` provides better output than `assert!(a == b)`:

```rust
let x = 1;
assert!(x == 2);
// Panic message:
// assertion failed: x == 2

assert_eq!(x, 2);
// Panic message:
// assertion `left == right` failed
//   left: 1
//  right: 2
```

As you can see, `assert_eq` is able to provide detailed info on what the individual values were.\
But: That doesn’t have to be the case. Rust has fancy-pants macros, so we can just
**make `assert!(a == b)` work the same as `assert_eq!(a, b)`:**

```rust
let x = 1;
one_assert::assert!(x == 2);
// Panic message:
// assertion `x == 2` failed
//      left: 1
//     right: 2
```

And now we can expand this to as many operators (and even expressions!) as we want.

### Examples

```rust
let x = 1;
one_assert::assert!(x > 2);
// assertion `x > 2` failed
//      left: 1
//     right: 2

one_assert::assert!(10 <= x);
// assertion `10 <= x` failed
//      left: 10
//     right: 1

one_assert::assert!(x != 1, "x ({}) should not be 1", x);
// assertion `x != 1` failed: x (1) should not be 1
//      left: 1
//     right: 1

let s = "Hello World";
one_assert::assert!(s.starts_with("hello"));
// assertion `s.starts_with("hello")` failed
//      self: "Hello World"
//     arg 0: "hello"
```

### Limitations
- **Several Components need to implement `Debug`**
  - The macro will take whatever part of the expression is considered useful and debug print it. This means that
    those parts need to implement `Debug`.
  - What is printed as part of any given expression type is subject to change, so it is recommended to only use
    this in code where pretty much everything implements `Debug`.
- **`Debug` printing might happen even if the assertion passes**
  - Because this macro prints more than just the two sides of an `==` or `!=` comparison, it has to deal with the
    fact that some values might be moved during the evaluation of the expression. This means that the values have
    to be printed in advance.
  - Specifically, comparisons work as usual, but every other operator that has special output (e.g. `a+b`,
    `foo(a,b)`, `arr[a]`, ...) has its arguments debug-printed in advance.
  - Consequence: **You might not want to use this macro in performance-critical code.**
  - Note however, that the expression and each part of it is only **evaluated** once, and the fail-fast behavior
    of `&&` and `||` is preserved.

### Changelog
See [Changelog.md](Changelog.md)

### License
Licensed under either of [Apache License, Version 2.0] or [MIT license] at your option.

[Apache License, Version 2.0]: LICENSE-APACHE
[MIT license]: LICENSE-MIT
