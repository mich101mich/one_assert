fn main() {
    let fut = async { 1 };
    let _ = async move {
        one_assert::assert!(fut.await);
    };
}
