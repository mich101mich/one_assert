fn main() {
    one_assert::assert!(true, "{}");
    one_assert::assert!(true "{}");
    one_assert::assert!(true, , "{}");
    one_assert::assert!(1 == 2, "{}", 1, 2);
}
