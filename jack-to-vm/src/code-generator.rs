#[path = "code-generator/symbols.rs"]
mod symbols;

use super::grammar::*;
use symbols::*;

/// Compile the class into a vector of VM command strings
pub fn compile(class: Class) -> Vec<String> {
    let code_generator = CodeGenerator::new(&class);
    code_generator.compile()
}

#[allow(dead_code)]
struct CodeGenerator<'a> {
    class: &'a Class,
    symbols: ClassSymbols,
}

impl<'a> CodeGenerator<'a> {
    fn new(class: &'a Class) -> Self {
        Self {
            class: class,
            symbols: ClassSymbols::new(&class.vars),
        }
    }

    fn compile(self: &Self) -> Vec<String> {
        vec![]
    }
}
