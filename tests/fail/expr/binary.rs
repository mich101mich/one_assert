fn main() {
    one_assert::assert!(1 + 1);
    one_assert::assert!(true == 1);
    let mut x = 0;
    one_assert::assert!(x += 1);
    let mut x = false;
    one_assert::assert!(x |= true);
}
