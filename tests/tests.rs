use project_castaway::hello_world;

#[test]
fn test_hello_world() {
    assert_eq!("Hello, world!", hello_world());
}
