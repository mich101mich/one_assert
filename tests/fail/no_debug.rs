fn main() {
    #[derive(PartialEq)]
    struct NoDebugImpl(i32);

    let x = NoDebugImpl(1);
    one_assert::assert!(x == NoDebugImpl(2));
}
