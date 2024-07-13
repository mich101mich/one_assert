fn main() {
    one_assert::assert!([1, 2][2 - 1]);             // wrong array type
    one_assert::assert!([1, 2][String::from("a")]); // wrong index type
    one_assert::assert!((1 + 2)[0]);                // not an array
}
