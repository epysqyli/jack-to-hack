use std::{env, fs, path::PathBuf};

use hack_assembler::assembler::Assembler;
use vm_translator::translate_vm_from_path;

fn main() {
    // vm -> asm -> hack
    let vm_program_path = env::args().nth(1).expect("No vm program path provided!");
    let vm_program_pathbuf = &PathBuf::from(vm_program_path.to_string());
    let asm_program = translate_vm_from_path(vm_program_pathbuf);

    let output_path = {
        let path = vm_program_pathbuf.file_name().unwrap().to_str().unwrap();
        if vm_program_pathbuf.is_dir() {
            path.to_string()
        } else {
            path.replace(".vm", "")
        }
    };

    if env::args().any(|arg| arg == "--with-asm") {
        fs::write(format!("{}.asm", output_path), asm_program.join("\n"))
            .expect("Writing .asm output failed");
    }

    match Assembler::new(asm_program).compile() {
        Ok(hack) => match fs::write(format!("{}.hack", output_path), hack.join("\n")) {
            Ok(_) => {
                let filename = vm_program_path.split("/").last().unwrap();
                println!("{filename} compiled to hack");
            }
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    }
}
