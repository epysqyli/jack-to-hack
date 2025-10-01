#[path = "syntax-analyzer/parser.rs"]
mod parser;
#[path = "syntax-analyzer/tokenizer.rs"]
mod tokenizer;

// Analyze the input stream and produce a grammatically
// valid target-neutral representation of the Jack program
pub fn run() {
    // TODO: tokenizer -> parser -> xml/json/yaml/rust objects output
}

#[cfg(test)]
mod tests {}
