fn main() {
    one_assert::assert!(1 == 2, "{}");
    one_assert::assert!(1 == 2 "{}");
    one_assert::assert!(1 == 2, , "{}");
    one_assert::assert!(1 == 2, "{}", 1, 2);
}
