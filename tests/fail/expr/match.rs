fn main() {
    one_assert::assert!(match 0 {
        1 => true,
        x => x + 1,
    });
}
