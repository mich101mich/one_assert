fn main() {
    let x = 1;
    one_assert::assert!(if x == 1 { 1 } else { 2 });
    one_assert::assert!(if x == 1 { 1 } else { 2 } else { 3 });
    one_assert::assert!(if x == 1 { true });
    one_assert::assert!(if x == 1 { true } else { 1 });
}
