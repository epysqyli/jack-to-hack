use std::{fs::read_to_string, path::PathBuf};

#[path = "asm-generator.rs"]
mod asm_generator;
mod command;
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

    let vm_instructions: Vec<String> = vm_file_paths
        .iter()
        .flat_map(|vm_file_path| read_vm_program_from_path(&vm_file_path))
        .collect();

    let commands = parser::parse(
        bootstrap_instructions
            .into_iter()
            .chain(vm_instructions)
            .collect(),
    );

    let mut asm = asm_generator::AsmGenerator::generate(commands);

    // Set SP to 256 as first bootstrapping step
    asm.insert(0, "@256".to_string());
    asm.insert(1, "D=A".to_string());
    asm.insert(2, "@SP".to_string());
    asm.insert(3, "M=D".to_string());

    asm
}
