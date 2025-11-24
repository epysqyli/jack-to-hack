#[test]
fn fibonacci_is_translated_correctly() {
    let expected: Vec<String> = std::fs::read_to_string("tests/fixtures/expected.asm")
        .expect("Cannot read expected.asm file")
        .split("\n")
        .map(|cmd| cmd.to_string())
        .collect();

    let vm_program =
        vm_translator::fetch_vm_program(&std::path::PathBuf::from("tests/fixtures/vm-program"));
    let actual = vm_translator::compile(vm_program);

    assert_eq!(expected, actual)
}
