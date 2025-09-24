use std::{env, fs, path::PathBuf};

use vm_translator::translate_vm_from_path;

fn main() {
    let vm_file_path = if env::current_dir().unwrap().ends_with("jack-to-hack") {
        "vm-translator/examples/push-and-add.vm"
    } else {
        "examples/push-and-add.vm"
    };

    let asm = translate_vm_from_path(&PathBuf::from(vm_file_path));

    fs::write(vm_file_path.replace(".vm", ".asm"), asm.join("\n"))
        .expect("Writing asm to file failed");
}
