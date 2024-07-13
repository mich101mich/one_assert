fn main() {
    one_assert::assert!(loop { break });
    one_assert::assert!(loop { break 1 });
    // one_assert::assert!(loop { }); // would be an error logically, but for compiler it just returns `!`, so it's not an error
}
