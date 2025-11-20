#[path = "syntax-analyzer/parser.rs"]
mod parser;
#[path = "syntax-analyzer/tokenizer.rs"]
mod tokenizer;

pub fn run(jack_class: String) -> super::grammar::Class {
    let tokens = tokenizer::tokenize(&jack_class);
    let derivation_tree = parser::parse(tokens);

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
                body: SubroutineBody { vars: vec![], statements: vec![Statement::Return(None)] },
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
                    vars: vec![VarDec { jack_type: JackType::Int, name: "a".to_owned() }],
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

    #[test]
    fn parse_class_with_contructor() {
        let input_program = r#"
            class Point {
                field int x, y;
                constructor Point new(int argX, int argY) {
                    let x = argX;
                    let y = argY;
                    return this;
                }
            }
        "#;

        let expected = Class {
            name: "Point".into(),
            vars: vec![
                ClassVarDec {
                    var_type: ClassVarType::Field,
                    jack_type: JackType::Int,
                    name: "x".into(),
                },
                ClassVarDec {
                    var_type: ClassVarType::Field,
                    jack_type: JackType::Int,
                    name: "y".into(),
                },
            ],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Constructor,
                return_type: ReturnType::Type(JackType::Class("Point".into())),
                name: "new".into(),
                parameters: vec![
                    Parameter { jack_type: JackType::Int, name: "argX".into() },
                    Parameter { jack_type: JackType::Int, name: "argY".into() },
                ],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![
                        Statement::Let {
                            var_name: "x".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::VarName("argX".into()),
                                additional: vec![],
                            },
                        },
                        Statement::Let {
                            var_name: "y".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::VarName("argY".into()),
                                additional: vec![],
                            },
                        },
                        Statement::Return(Some(Expression {
                            term: Term::KeywordConst("this".into()),
                            additional: vec![],
                        })),
                    ],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }

    #[test]
    fn parse_class_with_if_statement() {
        let input_program = r#"
            class Example {
                function int run(int a) {
                    if (a < 2) {
                        return a;
                    } else {
                        return a + 1;
                    }
                }
            }
        "#;

        let expected = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "run".into(),
                parameters: vec![Parameter { name: "a".into(), jack_type: JackType::Int }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::If {
                        exp: Expression {
                            term: Term::VarName("a".into()),
                            additional: vec![(Operation::LessThan, Term::IntConst(2))],
                        },
                        statements: vec![Statement::Return(Some(Expression {
                            term: Term::VarName("a".into()),
                            additional: vec![],
                        }))],
                        else_statements: Some(vec![Statement::Return(Some(Expression {
                            term: Term::VarName("a".into()),
                            additional: vec![(Operation::Plus, Term::IntConst(1))],
                        }))]),
                    }],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }

    #[test]
    fn parse_return_with_expression() {
        let input_program = r#"
            class Example {
                function int run(int n) {
                    return n + 1;
                }
            }
        "#;

        let expected = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "run".into(),
                parameters: vec![Parameter { name: "n".into(), jack_type: JackType::Int }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::VarName("n".into()),
                        additional: vec![(Operation::Plus, Term::IntConst(1))],
                    }))],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }

    #[test]
    fn parse_return_with_expression_within_parentheses() {
        let input_program = r#"
            class Example {
                function int run(int n) {
                    return (n + 1);
                }
            }
        "#;

        let expected = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "run".into(),
                parameters: vec![Parameter { name: "n".into(), jack_type: JackType::Int }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::Expression(Box::new(Expression {
                            term: Term::VarName("n".into()),
                            additional: vec![(Operation::Plus, Term::IntConst(1))],
                        })),
                        additional: vec![],
                    }))],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }

    #[test]
    fn parse_return_with_complex_exp_within_parentheses() {
        let input_program = r#"
            class Example {
                function int test() {
                    return ((1 + 2) + 3);
                }
            }
        "#;

        let expected = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "test".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::Expression(Box::new(Expression {
                            term: Term::Expression(Box::new(Expression {
                                term: Term::IntConst(1),
                                additional: vec![(Operation::Plus, Term::IntConst(2))],
                            })),
                            additional: vec![(Operation::Plus, Term::IntConst(3))],
                        })),
                        additional: vec![],
                    }))],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }

    #[test]
    fn parse_fibonacci() {
        let input_program = r#"
            class Fibonacci {
                function int run(int n) {
                    if (n < 2) {
                        return n;
                    } else {
                        return (Fibonacci.run(n - 2) + Fibonacci.run(n - 1));
                    }
                }
            }
        "#;

        let expected = Class {
            name: "Fibonacci".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "run".into(),
                parameters: vec![Parameter { name: "n".into(), jack_type: JackType::Int }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::If {
                        exp: Expression {
                            term: Term::VarName("n".into()),
                            additional: vec![(Operation::LessThan, Term::IntConst(2))],
                        },
                        statements: vec![Statement::Return(Some(Expression {
                            term: Term::VarName("n".into()),
                            additional: vec![],
                        }))],
                        else_statements: Some(vec![Statement::Return(Some(Expression {
                            term: Term::Expression(Box::new(Expression {
                                term: Term::Call(SubroutineCall {
                                    callee: Some("Fibonacci".into()),
                                    routine_name: "run".into(),
                                    expressions: vec![Expression {
                                        term: Term::VarName("n".into()),
                                        additional: vec![(Operation::Minus, Term::IntConst(2))],
                                    }],
                                }),
                                additional: vec![(
                                    Operation::Plus,
                                    Term::Call(SubroutineCall {
                                        callee: Some("Fibonacci".into()),
                                        routine_name: "run".into(),
                                        expressions: vec![Expression {
                                            term: Term::VarName("n".into()),
                                            additional: vec![(Operation::Minus, Term::IntConst(1))],
                                        }],
                                    }),
                                )],
                            })),
                            additional: vec![],
                        }))]),
                    }],
                },
            }],
        };

        assert_eq!(expected, super::run(input_program.into()));
    }

    #[test]
    fn parse_average_program() {
        let input = r#"
            class Main {
                function void main() {
                    var Array a;
                    var int length;
                    var int i, sum;

                    let length = Keyboard.readInt("How many numbers? ");
                    let a = Array.new(length); // constructs the array

                    let i = 0;
                    while (i < length) {
                        let a[i] = Keyboard.readInt("Enter a number: ");
                        let sum = sum + a[i];
                        let i = i + 1;
                    }

                    do Output.printString("The average is ");
                    do Output.printInt(sum / length);
                    return;
                }
            }
        "#;

        let expected = Class {
            name: "Main".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "main".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![
                        VarDec { jack_type: JackType::Class("Array".into()), name: "a".into() },
                        VarDec { jack_type: JackType::Int, name: "length".into() },
                        VarDec { jack_type: JackType::Int, name: "i".into() },
                        VarDec { jack_type: JackType::Int, name: "sum".into() },
                    ],
                    statements: vec![
                        Statement::Let {
                            var_name: "length".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::Call(SubroutineCall {
                                    callee: Some("Keyboard".into()),
                                    routine_name: "readInt".into(),
                                    expressions: vec![Expression {
                                        term: Term::StrConst("How many numbers? ".into()),
                                        additional: vec![],
                                    }],
                                }),
                                additional: vec![],
                            },
                        },
                        Statement::Let {
                            var_name: "a".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::Call(SubroutineCall {
                                    callee: Some("Array".into()),
                                    routine_name: "new".into(),
                                    expressions: vec![Expression {
                                        term: Term::VarName("length".into()),
                                        additional: vec![],
                                    }],
                                }),
                                additional: vec![],
                            },
                        },
                        Statement::Let {
                            var_name: "i".into(),
                            array_access: None,
                            exp: Expression { term: Term::IntConst(0), additional: vec![] },
                        },
                        Statement::While {
                            exp: Expression {
                                term: Term::VarName("i".into()),
                                additional: vec![(
                                    Operation::LessThan,
                                    Term::VarName("length".into()),
                                )],
                            },
                            statements: vec![
                                Statement::Let {
                                    var_name: "a".into(),
                                    array_access: Some(Expression {
                                        term: Term::VarName("i".into()),
                                        additional: vec![],
                                    }),
                                    exp: Expression {
                                        term: Term::Call(SubroutineCall {
                                            callee: Some("Keyboard".into()),
                                            routine_name: "readInt".into(),
                                            expressions: vec![Expression {
                                                term: Term::StrConst("Enter a number: ".into()),
                                                additional: vec![],
                                            }],
                                        }),
                                        additional: vec![],
                                    },
                                },
                                Statement::Let {
                                    var_name: "sum".into(),
                                    array_access: None,
                                    exp: Expression {
                                        term: Term::VarName("sum".into()),
                                        additional: vec![(
                                            Operation::Plus,
                                            Term::ArrayAccess {
                                                var_name: "a".into(),
                                                exp: Box::new(Expression {
                                                    term: Term::VarName("i".into()),
                                                    additional: vec![],
                                                }),
                                            },
                                        )],
                                    },
                                },
                                Statement::Let {
                                    var_name: "i".into(),
                                    array_access: None,
                                    exp: Expression {
                                        term: Term::VarName("i".into()),
                                        additional: vec![(Operation::Plus, Term::IntConst(1))],
                                    },
                                },
                            ],
                        },
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printString".into(),
                            expressions: vec![Expression {
                                term: Term::StrConst("The average is ".into()),
                                additional: vec![],
                            }],
                        }),
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printInt".into(),
                            expressions: vec![Expression {
                                term: Term::VarName("sum".into()),
                                additional: vec![(
                                    Operation::Divide,
                                    Term::VarName("length".into()),
                                )],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        assert_eq!(expected, super::run(input.into()));
    }

    #[test]
    fn compile_multiple_functions() {
        let input = r#"
            class Main {
                function void main() {
                    return;
                }

                function int first() {
                    return 1;
                }

                function void second() {
                    return;
                }
            }
        "#;

        assert_eq!(3, super::run(input.into()).routines.len());
    }
}
