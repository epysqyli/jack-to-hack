use std::{env, fs};

fn main() {
    let mut asm_file_path = "examples/fill-screen.asm";
    let mut hack_file_path = "examples/fill-screen.hack";

    if env::current_dir().unwrap().ends_with("jack-to-hack") {
        asm_file_path = "hack-assembler/examples/fill-screen.asm";
        hack_file_path = "hack-assembler/examples/fill-screen.hack";
    }

    match hack_assembler::assembler::compile_from_file(asm_file_path) {
        Ok(hack) => match fs::write(hack_file_path, hack.join("\n")) {
            Ok(_) => println!("fill-screen compiled to hack"),
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    }
}
