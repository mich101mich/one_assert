fn main() {
    struct Bob {
        age: u32,
    }
    let bob = Bob { age: 35 };
    one_assert::assert!(bob.age);
}
