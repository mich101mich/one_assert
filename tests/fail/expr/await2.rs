fn main() {
    let fut = async { true };
    one_assert::assert!(fut.await);
}
