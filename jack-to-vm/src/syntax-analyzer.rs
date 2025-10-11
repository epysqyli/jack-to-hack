#[path = "syntax-analyzer/parser.rs"]
mod parser;
#[path = "syntax-analyzer/tokenizer.rs"]
mod tokenizer;

// Analyze the input stream and produce a grammatically valid representation of the Jack program,
// i.e. the derivation tree needed as input for the code generation step
pub fn run() {
    // TODO: implement syntax analysis steps
    // - tokenizer -> read .jack file or all *.jack files from a directory
    // - parser -> read token stream, output xml/json/yaml/rust derivation tree
}

#[cfg(test)]
mod tests {}
