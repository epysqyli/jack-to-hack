use std::fs;

#[test]
fn compilation_outputs_expected_hack_program() {
    let expected_hack: Vec<String> = fs::read_to_string("tests/fixtures/mult.hack")
        .unwrap()
        .lines()
        .map(|l| l.to_string())
        .collect();

    let hack_output =
        hack_assembler::assembler::compile_from_file("tests/fixtures/mult.asm").unwrap();

    assert_eq!(expected_hack, hack_output);
}
