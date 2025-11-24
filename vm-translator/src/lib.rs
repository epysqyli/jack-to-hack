use std::{fs::read_to_string, path::PathBuf};

#[path = "asm-generator.rs"]
mod asm_generator;
mod command;
mod parser;

pub fn compile(vm_instructions: Vec<Vec<String>>) -> Vec<String> {
    // TODO: improve bootstrap when necessary.
    // Should @LCL, @ARG, @THIS, @THAT be initialized?
    let bootstrap_instructions = vec![
        "call Sys.init 0".to_string(),
        "label INFINITE_LOOP".to_string(),
        "goto INFINITE_LOOP".to_string(),
        "function Sys.init 0".to_string(),
        "call Main.main 0".to_string(),
        "return".to_string(),
    ];

    let instructions: Vec<String> = vm_instructions.into_iter().flat_map(|vm| vm).collect();
    let commands = parser::parse(bootstrap_instructions.into_iter().chain(instructions).collect());

    asm_generator::AsmGenerator::generate(commands)
}

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

pub fn fetch_vm_program(vm_path: &PathBuf) -> Vec<Vec<String>> {
    let mut vm_file_paths: Vec<PathBuf> = vec![];

    if vm_path.is_dir() {
        if let Ok(dir) = vm_path.read_dir() {
            for entry in dir {
                if let Ok(dir_entry) = entry {
                    if let Some(ext) = dir_entry.path().extension() {
                        if ext == "vm" {
                            vm_file_paths.push(dir_entry.path());
                        }
                    }
                }
            }
        }
    } else {
        vm_file_paths.push(vm_path.to_path_buf());
    }

    vm_file_paths.iter().map(|vm_file_path| read_vm_program_from_path(&vm_file_path)).collect()
}
