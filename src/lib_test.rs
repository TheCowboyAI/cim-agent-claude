//! Simple test to verify test framework is working

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(2 + 2, 4);
    }
    
    #[test]
    fn test_string_operations() {
        let hello = "hello";
        let world = "world";
        let combined = format!("{} {}", hello, world);
        assert_eq!(combined, "hello world");
    }
}