fn main() {
    one_assert::assert!("".to_string().len());
    one_assert::assert!("".len(1));
    one_assert::assert!(1.len());
}
