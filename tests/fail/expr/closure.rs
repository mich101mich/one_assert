fn main() {
    one_assert::assert!(|| true);
    one_assert::assert!(|| 5);
    one_assert::assert!(|x: usize| x + 5);
}
