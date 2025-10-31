#[path = "syntax-analyzer/parser.rs"]
mod parser;
#[path = "syntax-analyzer/tokenizer.rs"]
mod tokenizer;

pub fn run(jack_class: String) -> super::grammar::Class {
    let tokens = tokenizer::tokenize(&jack_class);
    let derivation_tree = parser::Parser::parse(tokens);

    derivation_tree
}

#[cfg(test)]
mod tests {
    use crate::grammar::*;

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
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
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

    #[test]
    fn parse_slightly_more_complex_class() {
        let input_program = r#"
            class Main {
                static int varA;

                function void main() {
                    var int a;
                    let a = varA + 1;
                    return;
                }
            }
        "#;

        let expected = Class {
            name: "Main".to_owned(),
            vars: vec![ClassVarDec {
                jack_type: JackType::Int,
                name: "varA".to_owned(),
                var_type: ClassVarType::Static,
            }],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "main".to_owned(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![VarDec {
                        jack_type: JackType::Int,
                        name: "a".to_owned(),
                    }],
                    statements: vec![
                        Statement::Let {
                            var_name: "a".to_owned(),
                            array_access: None,
                            exp: Expression {
                                term: Term::VarName("varA".to_owned()),
                                additional: vec![(Operation::Plus, Term::IntConst(1))],
                            },
                        },
                        Statement::Return(None),
                    ],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }
}
