### Jack Compiler
Jack is a Java-like high level language built from the ground up in the [From Nand To Tetris Course](https://www.nand2tetris.org/course).

This repo is organized in several crates:
- `hack-assembler`: an assembler from .asm to .hack machine language code
- `vm-translator`: intermediate stack machine language translator, from .vm files to .asm
- `jack-to-vm`: compiler from Jack to intermediate VM
- `compiler`: this is the executable, containing the jack OS classes too


### Example usage

Running the following command 
```bash
cargo run $DIR_CONTAINING_JACK_CLASSES (--with-vm) (--with-asm)
```
will produce a `source.hack` output which can be fed to the [nand to tetris CPU emulator](https://nand2tetris.github.io/web-ide/cpu)
