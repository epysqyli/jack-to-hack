use std::{env, fs, path::PathBuf};

use hack_assembler::assembler::Assembler;
use vm_translator::translate_vm_from_path;

fn main() {
    // vm -> asm -> hack
    let vm_program_path = env::args().nth(1).expect("No vm program path provided!");
    let vm_program_pathbuf = &PathBuf::from(vm_program_path.to_string());
    let asm_program = translate_vm_from_path(vm_program_pathbuf);

    let current_dir = env::current_dir().unwrap();
    let output_path = format!("{}/source", current_dir.to_str().unwrap());

    if env::args().any(|arg| arg == "--with-asm") {
        fs::write(format!("{}.asm", output_path), asm_program.join("\n"))
            .expect("Writing .asm output failed");
    }

    match Assembler::new(asm_program).compile() {
        Ok(hack) => match fs::write(format!("{}.hack", output_path), hack.join("\n")) {
            Ok(_) => println!("Compilation to hack successful"),
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    }
}
