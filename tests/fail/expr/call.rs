fn main() {
    fn test_fn() -> bool {
        true
    }
    fn param_fn(a: i32, b: i32) -> bool {
        a == b
    }
    fn curry() -> fn(i32, i32) -> bool {
        param_fn
    }
    fn call_fn(f: fn() -> bool) -> bool {
        f()
    }
    fn int_fn(i: i32) -> i32 {
        i
    }

    one_assert::assert!(test_fn);            // not called
    one_assert::assert!(test_fn(1));         // too many arguments
    one_assert::assert!(param_fn(1));        // not enough arguments
    one_assert::assert!(param_fn(1, 2, 3));  // too many arguments
    one_assert::assert!(param_fn("a", "b")); // wrong type
    one_assert::assert!(curry());            // returned function not called
    one_assert::assert!(call_fn(test_fn));   // fn pointer does not implement Debug
    one_assert::assert!(int_fn(1 + 1));           // wrong return type
}
