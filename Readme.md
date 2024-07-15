A Rust crate with a more powerful `assert!()` macro

[![Tests](https://github.com/mich101mich/one_assert/actions/workflows/test.yml/badge.svg)](https://github.com/mich101mich/one_assert/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/one_assert.svg)](https://crates.io/crates/one_assert)
[![Documentation](https://docs.rs/one_assert/badge.svg)](https://docs.rs/one_assert/)
[![Dependency status](https://deps.rs/repo/github/mich101mich/one_assert/status.svg)](https://deps.rs/repo/github/mich101mich/one_assert)

# One Assert

## TL;DR
Why have separate macros for `assert_eq` and `assert_ne` (and `assert_gt` etc. with other crates) when you can just get the same output with `assert!(a == b)` (or `assert!(a != b)`, `assert!(a > b)`, …)? This crate provides a single assert! macro that analyzes the expression to provide more detailed output on failure.

## Introduction
Rust’s standard library provides the `assert`, `assert_eq` and `assert_ne` macros. There are however some inconveniences with these, like how there are no specialization for other inequalities, like `assert_ge` for `>=` etc, or how the names only differ in one or two letters (`assert_eq`, `assert_ne`, `assert_ge`, `assert_gt`, …) and are thus easy to mix up at a glance.

The main reason for not adding more macros is that they can be represented just fine with `assert!(a >= b)`, so there is no need for a separate macro for every use case.

But that begs the question: Why do we have `assert_eq` and `assert_ne` in the first place?

The practical reason: `assert_eq!(a, b)` provides better output than `assert!(a == b)`:

```rust
let x = 1;
let msg = catch_panic!({ assert!(x == 2); });
assert_eq!(msg, "assertion failed: x == 2");

let msg = catch_panic!({ assert_eq!(x, 2); });
assert_eq!(msg, "assertion `left == right` failed
  left: 1
 right: 2"
);
```
As you can see, `assert_eq` is able to provide detailed info on what the individual values were.
But: That doesn’t have to be the case. Rust has hygienic and procedural macros, so we can just make `assert!(a == b)` work the same as `assert_eq!(a, b)`:

```rust
let x = 1;
let msg = catch_panic!({ one_assert::assert!(x == 2); });
assert_eq!(msg, "assertion `x == 2` failed
     left: 1
    right: 2"
);
```
And now we can expand this to as many operators as we want:

```rust
let x = 1;
let msg = catch_panic!({ one_assert::assert!(x > 2); });
assert_eq!(msg, "assertion `x > 2` failed
     left: 1
    right: 2"
);
```

## Examples
```rust
let x = 1;
let msg = catch_panic!({ one_assert::assert!(x > 2); });
assert_eq!(msg, "assertion `x > 2` failed
     left: 1
    right: 2"
);

let msg = catch_panic!({ one_assert::assert!(x != 1, "x ({}) should not be 1", x); });
assert_eq!(msg, "assertion `x != 1` failed: x (1) should not be 1
     left: 1
    right: 1"
);

let s = "Hello World";
let msg = catch_panic!({ one_assert::assert!(s.starts_with("hello")); });
assert_eq!(msg, r#"assertion `s.starts_with("hello")` failed
     self: "Hello World"
    arg 0: "hello""#
);
```
Limitations
- **Several Components need to implement `Debug`**
  - The macro will take whatever part of the expression is considered useful and debug print it. This means that those parts need to implement `Debug`.
  - What is printed as part of any given expression type is subject to change, so it is recommended to only use this in code where pretty much everything implements `Debug`.
- **`Debug` printing happens even if the assertion passes**
  - Because this macro prints more than just the two sides of an `==` or `!=` comparison, it has to deal with the fact that some values are moved during the evaluation of the expression. This means that the values have to be printed in advance.
  - Consequence: **Don’t use this macro in performance-critical code.**
  - Note however, that the expression and each part of it is only **evaluated** once.
    - (Though it is worth noting that fail-fast operators like `&&` might normally only evaluate the left side and stop, but with this macro it will always evaluate both sides)