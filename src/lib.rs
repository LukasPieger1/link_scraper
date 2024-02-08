pub mod parser;

/// # Examples
/// ```
/// use untitled_rust_parser::add; // TODO ask felix about import
/// let result = add(2, 2);
/// assert_eq!(result, 4);
/// ```
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

/// # Examples
/// ```
/// use untitled_rust_parser::sub;
/// let result = sub(4,2);
/// println!("Result is {}", result); // TODO Where does this output go?
/// assert_eq!(result, 2)
/// ```
pub fn sub (left: i32, right: i32) -> i32{
    add(left, -right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
