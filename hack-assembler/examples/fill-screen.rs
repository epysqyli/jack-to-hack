use std::fs;

use hack_assembler::assembler::Assembler;

fn main() {
    match Assembler::from_file("hack-assembler/examples/fill-screen.asm") {
        Ok(assembler) => match assembler.compile() {
            Ok(hack) => match fs::write("hack-assembler/examples/fill-screen.hack", hack.join("\n")) {
                Ok(_) => println!("fill-screen compiled to hack"),
                Err(err) => eprintln!("{err}"),
            },
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    };
}
