use std::{collections::HashMap, env, fs, path::PathBuf};

/* jack -> vm -> asm -> hack */
fn main() {
    let program_path = env::args().nth(1).expect("No program path provided!");
    let program_pathbuf = &PathBuf::from(program_path.to_string());
    let vm_instructions = jack_to_vm::compile(program_pathbuf);

    // TODO: fix path or avoid compilation from jack to vm to begin with
    // and provide the vm versions as part of the bootstrapping process.
    let os_path = &PathBuf::from("compiler/jack-os");
    let jack_os_instructions = jack_to_vm::compile(os_path);

    let program_and_os: HashMap<String, Vec<String>> =
        vm_instructions.into_iter().chain(jack_os_instructions).collect();

    if env::args().any(|arg| arg == "--with-vm") {
        program_and_os.iter().for_each(|(name, vm)| {
            fs::write(format!("{}.vm", name.replace(".jack", "")), vm.join("\n"))
                .expect("Writing .vm output failed");
        });
    }

    let asm_program = vm_translator::compile(program_and_os.into_values().collect());
    let current_dir = env::current_dir().unwrap();
    let output_path = format!("{}/source", current_dir.to_str().unwrap());

    if env::args().any(|arg| arg == "--with-asm") {
        fs::write(format!("{}.asm", output_path), asm_program.join("\n"))
            .expect("Writing .asm output failed");
    }

    match hack_assembler::assembler::compile(asm_program) {
        Ok(hack) => match fs::write(format!("{}.hack", output_path), hack.join("\n")) {
            Ok(_) => println!("Compilation to hack successful"),
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    }
}
