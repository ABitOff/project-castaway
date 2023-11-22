mod vulkan;

#[test]
fn test() {
    let compiler = shaderc::Compiler::new();
    assert!(compiler.is_some());
}
