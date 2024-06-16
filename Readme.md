A Rust crate with a more powerful `assert!()` macro

[![Tests](https://github.com/mich101mich/one_assert/actions/workflows/test.yml/badge.svg)](https://github.com/mich101mich/one_assert/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/one_assert.svg)](https://crates.io/crates/one_assert)
[![Documentation](https://docs.rs/one_assert/badge.svg)](https://docs.rs/one_assert/)
[![Dependency status](https://deps.rs/repo/github/mich101mich/one_assert/status.svg)](https://deps.rs/repo/github/mich101mich/one_assert)

# One Assert

### Introduction

Rust's standard library provides the [`assert`](https://doc.rust-lang.org/std/macro.assert.html), [`assert_eq`](https://doc.rust-lang.org/std/macro.assert_eq.html) and [`assert_ne`](https://doc.rust-lang.org/std/macro.assert_ne.html). There are however some inconveniences with these, like how the `eq` and `ne` variants can look pretty similar with long expressions after them, or how there are no specialization for other inequalities, like `assert_ge` for `>=` etc.  
The main reason for not adding more macros is, that they can be represented just fine with `assert!(a >= b)`, so there is no need for a separate macro for every use case. But that begs the question: Why do we have `assert_eq` and `assert_ne` in the first place?  
The practical reason: `assert_eq!(a, b)` provides better output than `assert!(a == b)`:
```rust

let panic_message = std::panic::catch_unwind(|| {

    let a = 1;
    let b = 2;
    assert!(a == b);

}).unwrap_err().downcast::<&'static str>().unwrap();
assert_eq!(
    *panic_message,
    "assertion failed: a == b"
);

let panic_message = std::panic::catch_unwind(|| {

    let a = 1;
    let b = 2;
    assert_eq!(a, b);

}).unwrap_err().downcast::<String>().unwrap();
assert_eq!(
    *panic_message,
    "assertion `left == right` failed
  left: 1
 right: 2"
);
```
As you can see, `assert_eq` is able to provide detailed info on what the individual values were.  
But: That doesn't have to be the case. Rust has hygienic and procedural macros, so we can just **make `assert!(a == b)` work the same as `assert_eq!(a, b)`**:
```rust
let panic_message = std::panic::catch_unwind(|| {

    let a = 1;
    let b = 2;
    one_assert::assert!(a == b);

}).unwrap_err().downcast::<String>().unwrap();
assert_eq!(
    *panic_message,
    "assertion `a == b` failed
  left: 1
 right: 2"
);
```
And now we can expand this to as many operators as we want:
```rust
let panic_message = std::panic::catch_unwind(|| {

    let a = 1;
    let b = 2;
    one_assert::assert!(a > b);

}).unwrap_err().downcast::<String>().unwrap();
assert_eq!(
    *panic_message,
    "assertion `a > b` failed
  left: 1
 right: 2"
);
```
