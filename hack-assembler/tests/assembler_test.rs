use std::fs;

use hack_assembler::assembler;

#[test]
fn assembler_is_initialized_from_file() {
    let assembler = assembler::Assembler::from_file("tests/fixtures/mult.asm");
    assert!(assembler.is_ok())
}

#[test]
fn compilation_outputs_expected_hack_program() {
    let expected_hack: Vec<String> = fs::read_to_string("tests/fixtures/mult.hack")
        .unwrap()
        .lines()
        .map(|l| l.to_string())
        .collect();

    let assembler = assembler::Assembler::from_file("tests/fixtures/mult.asm").unwrap();

    assert_eq!(expected_hack, assembler.compile().unwrap());
}
