use std::{env, fs, path::PathBuf};

use vm_translator::translate_vm_from_path;

fn main() {
    let vm_file_path = if env::current_dir().unwrap().ends_with("jack-to-hack") {
        "vm-translator/examples/multiple-functions"
    } else {
        "examples/multiple-functions"
    };

    let asm = translate_vm_from_path(&PathBuf::from(vm_file_path));

    fs::write("source.asm", asm.join("\n")).expect("Writing asm to file failed");
}
