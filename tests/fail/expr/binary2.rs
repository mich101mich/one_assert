fn main() {
    let mut x = 0;
    one_assert::assert!(x += 1);
    let mut x = false;
    one_assert::assert!(x |= true);
}
