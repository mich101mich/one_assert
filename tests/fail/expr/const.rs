fn main() {
    one_assert::assert!(const { 3 });
    one_assert::assert!(
        const {
            let x = 1;
            x == 1;
        }
    );
}
