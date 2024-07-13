fn main() {
    struct S {
        b: bool,
    }
    one_assert::assert!(S { b: true });
}
