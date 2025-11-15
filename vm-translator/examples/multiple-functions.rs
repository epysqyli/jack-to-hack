use std::{env, fs, path::PathBuf};

fn main() {
    let vm_file_path = if env::current_dir().unwrap().ends_with("jack-to-hack") {
        "vm-translator/examples/multiple-functions"
    } else {
        "examples/multiple-functions"
    };

    let vm_program = vm_translator::fetch_vm_program(&PathBuf::from(vm_file_path));
    let asm = vm_translator::compile_vm_to_asm(vm_program);

    fs::write("source.asm", asm.join("\n")).expect("Writing asm to file failed");
}
