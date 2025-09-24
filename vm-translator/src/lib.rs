use crate::{asm_generator::generate_asm, parser::parse};
use std::{fs::read_to_string, path::PathBuf};

#[path = "asm-generator.rs"]
mod asm_generator;
mod parser;

fn read_vm_program_from_path(vm_program_path: &PathBuf) -> Vec<String> {
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

pub fn translate_vm_from_path(vm_path: &PathBuf) -> Vec<String> {
    let mut vm_file_paths: Vec<PathBuf> = vec![];

    if vm_path.is_dir() {
        match vm_path.read_dir() {
            Ok(dir) => {
                for entry in dir {
                    if let Ok(dir_entry) = entry {
                        vm_file_paths.push(dir_entry.path());
                    }
                }
            }
            Err(e) => panic!("{e}"),
        }
    } else {
        vm_file_paths.push(vm_path.to_path_buf());
    }

    // TODO: bootstrap hack machine:
    // - set SP to 256
    // - call Sys.init, which calls Main.main
    let mut asm: Vec<String> = vec![];

    for vm_file_path in vm_file_paths {
        let program_name = &vm_file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .collect::<Vec<&str>>()[0];

        let vm_program = read_vm_program_from_path(&vm_file_path);
        let vm_commands = parse(vm_program);
        let program_asm = generate_asm(vm_commands, &program_name);

        program_asm
            .into_iter()
            .for_each(|asm_command| asm.push(asm_command));
    }

    // TODO: should this be done?
    asm.push("(END)".to_string());
    asm.push("@END".to_string());
    asm.push("0;JMP".to_string());

    asm
}
