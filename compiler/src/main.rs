use std::{env, fs, path::PathBuf};

fn main() {
    let program_path = env::args().nth(1).expect("No program path provided!");
    let program_pathbuf = &PathBuf::from(program_path.to_string());
    let vm_program = jack_to_vm::compile(program_pathbuf);

    if env::args().any(|arg| arg == "--with-vm") {
        vm_program.iter().for_each(|(name, vm)| {
            fs::write(format!("{}.vm", name.replace(".jack", "")), vm.join("\n"))
                .expect("Writing .vm output failed");
        });
    }

    let mut ordered_vm = vec![];
    let vm_os = jack_to_vm::compile(&PathBuf::from("compiler/jack-os"));

    ordered_vm.push(vm_os.get("compiler/jack-os/Sys.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/Memory.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/Array.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/Output.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/Math.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/Screen.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/String.jack").unwrap().clone());
    ordered_vm.push(vm_os.get("compiler/jack-os/Keyboard.jack").unwrap().clone());

    vm_program.into_values().for_each(|class| ordered_vm.push(class));

    let asm_program = vm_translator::compile(ordered_vm);
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
