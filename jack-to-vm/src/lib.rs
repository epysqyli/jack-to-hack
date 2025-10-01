#[path = "syntax-analyzer.rs"]
mod syntax_analyzer;

// TODO: implement
// tokenizer -> parser -> code generator
pub fn compile() {
    syntax_analyzer::run();
}

#[cfg(test)]
mod tests {}
