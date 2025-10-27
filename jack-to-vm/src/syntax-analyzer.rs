use crate::syntax_analyzer::grammar::Class;

mod grammar;
#[path = "syntax-analyzer/parser.rs"]
mod parser;
#[path = "syntax-analyzer/tokenizer.rs"]
mod tokenizer;

pub fn run(jack_class: String) -> Class {
    let tokens = tokenizer::tokenize(&jack_class);
    let derivation_tree = parser::Parser::parse(tokens);

    derivation_tree
}

#[cfg(test)]
mod tests {
    use crate::syntax_analyzer::grammar::*;

    #[test]
    fn parse_mininal_class() {
        let input_program = r#"
            class Main {
                function void main() {
                    return;
                }
            }
        "#;

        let expected = Class {
            name: "Main".to_owned(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: "function".to_owned(),
                return_type: "void".to_owned(),
                name: "main".to_owned(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(None)],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }
}
