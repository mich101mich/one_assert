fn main() {
    // the other match.rs file checks for types and stops checking, so this file is needed to check for later errors
    one_assert::assert!(match 0 {});
}
