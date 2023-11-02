pub fn hello_world() -> &'static str {
    return "Hello, world!";
}

#[test]
fn test_hello_world() {
    assert_eq!("Hello, world!", hello_world());
}
