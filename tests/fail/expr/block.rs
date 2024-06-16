fn main() {
    one_assert::assert!({
        let x = 5;
        x + 1
    });
    one_assert::assert!({
        let x = 5;
        x == 5;
    });
    one_assert::assert!({
        let x = 5;
    });
}
