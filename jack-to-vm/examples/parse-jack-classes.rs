use std::{env, path::PathBuf};

fn main() {
    let path = if env::current_dir().unwrap().ends_with("jack-to-hack") {
        "jack-to-vm/examples/program"
    } else {
        "examples/program"
    };

    jack_to_vm::compile(&PathBuf::from(path));
}
