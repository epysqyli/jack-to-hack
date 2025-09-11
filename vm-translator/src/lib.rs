use crate::{asm_generator::generate_asm, parser::parse};
use std::{
    fs::{File, read_to_string},
    io::Write,
};

#[path = "asm-generator.rs"]
mod asm_generator;
mod parser;

fn read_vm_program_from_file(vm_program_path: &str) -> Vec<String> {
    match read_to_string(vm_program_path) {
        Err(err) => panic!("{err}"),
        Ok(vm_program) => vm_program
            .lines()
            .map(|l| l.split("//").collect::<Vec<&str>>()[0])
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect::<Vec<String>>(),
    }
}

fn parse_vm_program(vm_program: Vec<String>, program_name: &str) -> Vec<String> {
    let mut asm_commands: Vec<String> = vm_program
        .iter()
        .flat_map(|vm_instruction| {
            let vm_command = parse(&vm_instruction);
            generate_asm(&vm_command, program_name)
        })
        .collect();

    asm_commands.push("(END)".to_string());
    asm_commands.push("@END".to_string());
    asm_commands.push("0;JMP".to_string());

    asm_commands
}

pub fn parse_vm_program_from_file(vm_program_path: &str) -> Vec<String> {
    let vm_program_path_segments: Vec<&str> = vm_program_path.split("/").collect();
    let vm_program_name: &str = vm_program_path_segments
        .last()
        .unwrap()
        .split(".")
        .collect::<Vec<&str>>()[0];

    let vm_program = read_vm_program_from_file(vm_program_path);
    parse_vm_program(vm_program, vm_program_name)
}

pub fn translate_vm_program_to_file(vm_program_path: &str) {
    let asm_instructions = parse_vm_program_from_file(vm_program_path);

    let mut asm_program_file = File::options()
        .create_new(true)
        .write(true)
        .append(true)
        .open(vm_program_path.replace(".vm", ".asm"))
        .unwrap();

    match writeln!(asm_program_file, "{}", asm_instructions.join("\n")) {
        Ok(_) => {}
        Err(err) => panic!("{err}"),
    }
}
