use std::env;

use vm_translator::translate_vm_program_to_file;

fn main() {
    let vm_program_path = env::args().nth(1).expect("No vm program path provided!");
    translate_vm_program_to_file(&vm_program_path);
}
