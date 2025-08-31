use std::{env, fs};

use hack_assembler::assembler::Assembler;
use vm_translator::parse_vm_program_from_file;

fn main() {
    // vm -> asm -> hack
    let vm_program_path = env::args().nth(1).expect("No vm program path provided!");
    let asm_program = parse_vm_program_from_file(&vm_program_path);

    match Assembler::new(asm_program).compile() {
        Ok(hack) => match fs::write(vm_program_path.replace(".vm", ".hack"), hack.join("\n")) {
            Ok(_) => {
                let filename = vm_program_path.split("/").last().unwrap();
                println!("{filename} compiled to hack");
            }
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    }
}
