fn main() {
    one_assert::assert!(5);
    one_assert::assert!(5, "hi");
    one_assert::assert!(1 + 2);
    one_assert::assert!(1 + 2, "hi");
}
