fn main() {
    let x = 1;
    one_assert::assert!(if x == 1 { 1 } else { 2 });            // not bool
    one_assert::assert!(if x == 1 { 1 } else { 2 } else { 3 }); // too many else
    one_assert::assert!(if x == 1 { true });                    // no else
    one_assert::assert!(if x == 1 { true } else { 1 });         // mismatched types
    one_assert::assert!(if x == 1 { true } else if true { false } else if false { true } ); // no final else
    one_assert::assert!(if x == 1 { true } else if true { false } else while false {} );    // else while instead of else if
}
