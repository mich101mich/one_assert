fn main() -> Result<(), ()> {
    one_assert::assert!(true?);
    one_assert::assert!(Ok(1)?);

    Ok(())
}
