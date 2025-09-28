use std::{fs, path::PathBuf};

#[test]
fn fibonacci_is_translated_correctly() {
    let expected: Vec<String> = fs::read_to_string("tests/fixtures/expected.asm")
        .expect("Cannot read expected.asm file")
        .split("\n")
        .map(|cmd| cmd.to_string())
        .collect();

    let actual = vm_translator::translate_vm_from_path(&PathBuf::from("tests/fixtures/vm-program"));

    assert_eq!(expected, actual)
}