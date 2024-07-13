fn main() {
    one_assert::assert!(return);
    fn with_value() -> i32 {
        one_assert::assert!(return 1);
        0
    }
}
