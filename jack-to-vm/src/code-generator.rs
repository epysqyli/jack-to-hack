#[path = "code-generator/symbols.rs"]
mod symbols;

use super::grammar::*;
use symbols::*;

pub fn compile(class: Class) -> Vec<String> {
    let mut code_generator = CodeGenerator::new(&class);
    code_generator.compile();
    code_generator.vm
}

struct CodeGenerator<'a> {
    class: &'a Class,
    class_symbols: ClassSymbols,
    routine_symbols: Option<RoutineSymbols>,
    label_counter: u16,
    vm: Vec<String>,
}

impl<'a> CodeGenerator<'a> {
    fn new(class: &'a Class) -> Self {
        Self {
            class: class,
            class_symbols: ClassSymbols::new(&class.vars),
            routine_symbols: None,
            label_counter: 0,
            vm: vec![],
        }
    }

    fn compile(self: &mut Self) {
        self.class.routines.iter().for_each(|routine| self.compile_routine(routine));
    }

    fn compile_routine(self: &mut Self, routine: &SubroutineDec) {
        self.vm.push(format!(
            "function {}.{} {}",
            self.class.name,
            routine.name,
            routine.body.vars.len()
        ));

        self.label_counter = 0;
        self.routine_symbols = Some(RoutineSymbols::new(routine, &self.class.name));

        match routine.routine_type {
            RoutineType::Constructor => {
                self.vm.push(format!("push constant {}", self.class_symbols.field_counter));
                self.vm.push("call Memory.alloc 1".into());
                self.vm.push("pop pointer 0".into());
            }
            RoutineType::Method => {
                self.vm.push("push argument 0".into());
                self.vm.push("pop pointer 0".into());
            }
            RoutineType::Function => {}
        }

        routine.body.statements.iter().for_each(|statement| self.compile_statement(statement));
    }

    fn compile_statement(self: &mut Self, statement: &Statement) {
        match statement {
            Statement::Let { var_name, array_access, exp } => match array_access {
                None => {
                    self.compile_expression(exp);
                    let entry = self.fetch_symbol_entry(var_name).unwrap();
                    self.vm.push(format!("pop {} {}", entry.kind.vm(), entry.index));
                }
                Some(index_exp) => {
                    let entry = self.fetch_symbol_entry(var_name).unwrap();
                    self.vm.push(format!("push {} {}", entry.kind.vm(), entry.index));
                    self.compile_expression(index_exp);
                    self.vm.push("add".into());
                    self.compile_expression(exp);
                    self.vm.push("pop temp 0".into());
                    self.vm.push("pop pointer 1".into());
                    self.vm.push("push temp 0".into());
                    self.vm.push("pop that 0".into());
                }
            },
            /* TODO: optimize branch handling setup ? */
            Statement::If { exp, statements, else_statements } => {
                let counter = self.label_counter;
                self.label_counter += 1;

                self.compile_expression(exp);
                self.vm.push(format!("if-goto IfTrue${}", counter));
                self.vm.push(format!("goto IfFalse${}", counter));

                /* handle TRUE branch */
                self.vm.push(format!("label IfTrue${}", counter));
                statements.iter().for_each(|s| self.compile_statement(s));
                self.vm.push(format!("goto IfDone${}", counter));

                /* handle the optional FALSE branch */
                self.vm.push(format!("label IfFalse${}", counter));
                if let Some(else_statements) = else_statements {
                    else_statements.iter().for_each(|s| self.compile_statement(s));
                }

                self.vm.push(format!("goto IfDone${}", counter));
                self.vm.push(format!("label IfDone${}", counter));
            }
            /* TODO: optimize branch handling setup ? */
            Statement::While { exp, statements } => {
                let counter = self.label_counter;
                self.label_counter += 1;

                /* define condition */
                self.vm.push(format!("label WhileCondition${}", counter));
                self.compile_expression(exp);

                /* execute the loop statements or break out of it */
                self.vm.push(format!("if-goto WhileStatements${}", counter));
                self.vm.push(format!("goto WhileDone${}", counter));
                self.vm.push(format!("label WhileStatements${}", counter));
                statements.iter().for_each(|statement| self.compile_statement(statement));
                self.vm.push(format!("goto WhileCondition${}", counter));

                /* resume execution after the while statement is complete */
                self.vm.push(format!("label WhileDone${}", counter));
            }
            Statement::Do(call) => {
                self.compile_routine_call(call);
                self.vm.push("pop temp 0".into());
            }
            Statement::Return(exp_opt) => match exp_opt {
                None => {
                    self.vm.push("push constant 0".into());
                    self.vm.push("return".into());
                }
                Some(exp) => {
                    self.compile_expression(exp);
                    self.vm.push("return".into());
                }
            },
        }
    }

    fn compile_expression(self: &mut Self, exp: &Expression) {
        self.compile_term(&exp.term);

        exp.additional.iter().for_each(|(op, term)| {
            self.compile_term(term);

            match op {
                Operation::Plus => self.vm.push("add".into()),
                Operation::Minus => self.vm.push("sub".into()),
                Operation::GreaterThan => self.vm.push("gt".into()),
                Operation::LessThan => self.vm.push("lt".into()),
                Operation::Equals => self.vm.push("eq".into()),
                Operation::And => self.vm.push("and".into()),
                Operation::Or => self.vm.push("or".into()),
                Operation::Not => self.vm.push("neg".into()),
                Operation::Multiply => self.vm.push("call Math.multiply 2".into()),
                Operation::Divide => self.vm.push("call Math.divide 2".into()),
            };
        });
    }

    fn compile_term(self: &mut Self, term: &Term) {
        match term {
            Term::IntConst(val) => {
                self.vm.push(format!("push constant {}", val));
            }
            Term::VarName(val) => {
                let entry = self.fetch_symbol_entry(val).unwrap();
                self.vm.push(format!("push {} {}", entry.kind.vm(), entry.index));
            }
            Term::Expression(exp) => {
                self.compile_expression(exp);
            }
            Term::Unary { op, term } => {
                self.compile_term(term);
                match op {
                    Operation::Minus => self.vm.push("neg".into()),
                    Operation::Not => self.vm.push("not".into()),
                    _ => panic!("Operation not supported for Term::Unary"),
                }
            }
            Term::Call(call) => self.compile_routine_call(call),
            Term::StrConst(val) => {
                self.vm.push(format!("push constant {}", val.len()));
                self.vm.push("call String.new 1".into());
                val.chars().for_each(|c| {
                    self.vm.push(format!("push constant {}", c as u8));
                    self.vm.push("call String.appendChar 2".into());
                });
            }
            Term::ArrayAccess { var_name, exp } => {
                let entry = self.fetch_symbol_entry(var_name).unwrap();
                self.vm.push(format!("push {} {}", entry.kind.vm(), entry.index));
                self.compile_expression(exp);
                self.vm.push("add".into());
                self.vm.push("pop pointer 1".into());
                self.vm.push("push that 0".into());
            }
            Term::KeywordConst(val) => match val.as_str() {
                "true" => {
                    self.vm.push("push constant 1".into());
                    self.vm.push("neg".into());
                }
                "false" => self.vm.push("push constant 0".into()),
                "this" => self.vm.push("push pointer 0".into()),
                "null" => self.vm.push("push constant 0".into()),
                _ => panic!("[WIP] Term::KeywordConst not handled"),
            },
        }
    }

    fn compile_routine_call(self: &mut Self, call: &SubroutineCall) {
        let (callee_name, n_args) = match &call.callee {
            Some(callee) => {
                if callee.chars().nth(0).unwrap().is_uppercase() {
                    /* function call */
                    (callee, call.expressions.len())
                } else {
                    /* method call on other object */
                    self.compile_term(&Term::VarName(callee.to_owned()));
                    let callee_entry = self.fetch_symbol_entry(callee).unwrap();
                    match &callee_entry.jtype {
                        JackType::Class(class) => (&class.to_owned(), call.expressions.len() + 1),
                        _ => panic!("Callee must be a Jack Type"),
                    }
                }
            }
            None => {
                /* method call on current object */
                self.compile_term(&Term::KeywordConst("this".into()));
                (&self.class.name, call.expressions.len() + 1)
            }
        };

        call.expressions.iter().for_each(|exp| self.compile_expression(exp));
        self.vm.push(format!("call {}.{} {}", callee_name, call.routine_name, n_args));
    }

    fn fetch_symbol_entry(self: &Self, varname: &String) -> Option<&SymbolEntry> {
        match self.routine_symbols.as_ref().unwrap().entries.get(varname) {
            Some(entry) => Some(entry),
            None => match self.class_symbols.entries.get(varname) {
                Some(entry) => Some(entry),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::grammar::*;

    #[test]
    fn compile_class_with_void_function() {
        /*
         * class Example {
         *     function void doNothing() {
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "doNothing".into(),
                parameters: vec![],
                body: SubroutineBody { vars: vec![], statements: vec![Statement::Return(None)] },
            }],
        };

        let expected = vec!["function Example.doNothing 0", "push constant 0", "return"];

        assert_eq!(expected, super::compile(class))
    }

    #[test]
    fn compile_simple_routine() {
        /*
         * class Example {
         *     function int incrTwice(int a) {
         *         var int res;
         *         let res = a + 1 + 1;
         *         return res;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "incrTwice".into(),
                parameters: vec![Parameter { jack_type: JackType::Int, name: "a".into() }],
                body: SubroutineBody {
                    vars: vec![VarDec { jack_type: JackType::Int, name: "res".into() }],
                    statements: vec![
                        Statement::Let {
                            var_name: "res".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![
                                    (Operation::Plus, Term::IntConst(1)),
                                    (Operation::Plus, Term::IntConst(1)),
                                ],
                            },
                        },
                        Statement::Return(Some(Expression {
                            term: Term::VarName("res".into()),
                            additional: vec![],
                        })),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.incrTwice 1",
            "push argument 0",
            "push constant 1",
            "add",
            "push constant 1",
            "add",
            "pop local 0",
            "push local 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class))
    }

    #[test]
    fn compile_simple_routine_with_static_var_access() {
        /*
         * class Example {
         *     static int a;
         *     function int incrStaticVarA() {
         *         var int res;
         *         let res = a + 1 + 1;
         *         return res;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![ClassVarDec {
                var_type: ClassVarType::Static,
                jack_type: JackType::Int,
                name: "a".into(),
            }],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "incrStaticVarA".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![VarDec { jack_type: JackType::Int, name: "res".into() }],
                    statements: vec![
                        Statement::Let {
                            var_name: "res".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![
                                    (Operation::Plus, Term::IntConst(1)),
                                    (Operation::Plus, Term::IntConst(1)),
                                ],
                            },
                        },
                        Statement::Return(Some(Expression {
                            term: Term::VarName("res".into()),
                            additional: vec![],
                        })),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.incrStaticVarA 1",
            /* let statement */
            "push static 0",
            "push constant 1",
            "add",
            "push constant 1",
            "add",
            "pop local 0",
            /* return statement */
            "push local 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class))
    }

    #[test]
    fn compile_simple_routine_without_local_var() {
        /*
         * class Example {
         *     function int incrTwice(int a) {
         *         return a + 1 + 1;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "incrTwice".into(),
                parameters: vec![Parameter { jack_type: JackType::Int, name: "a".into() }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::VarName("a".into()),
                        additional: vec![
                            (Operation::Plus, Term::IntConst(1)),
                            (Operation::Plus, Term::IntConst(1)),
                        ],
                    }))],
                },
            }],
        };

        let expected = vec![
            "function Example.incrTwice 0",
            /* return statement */
            "push argument 0",
            "push constant 1",
            "add",
            "push constant 1",
            "add",
            "return",
        ];

        assert_eq!(expected, super::compile(class))
    }

    #[test]
    fn compile_simple_routine_with_args_only() {
        /*
         * class Example {
         *     function int sumArgs(int a, int b, int c) {
         *         return a + b + c;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "sumArgs".into(),
                parameters: vec![
                    Parameter { jack_type: JackType::Int, name: "a".into() },
                    Parameter { jack_type: JackType::Int, name: "b".into() },
                    Parameter { jack_type: JackType::Int, name: "c".into() },
                ],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::VarName("a".into()),
                        additional: vec![
                            (Operation::Plus, Term::VarName("b".into())),
                            (Operation::Plus, Term::VarName("c".into())),
                        ],
                    }))],
                },
            }],
        };

        let expected = vec![
            "function Example.sumArgs 0",
            /* return statement */
            "push argument 0",
            "push argument 1",
            "add",
            "push argument 2",
            "add",
            "return",
        ];

        assert_eq!(expected, super::compile(class))
    }

    #[test]
    fn compile_routine_with_unary_exp() {
        /*
         * class Example {
         *     function int negate(int a) {
         *         return -a;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "negate".into(),
                parameters: vec![Parameter { jack_type: JackType::Int, name: "a".into() }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::Unary {
                            op: Operation::Minus,
                            term: Box::new(Term::VarName("a".into())),
                        },
                        additional: vec![],
                    }))],
                },
            }],
        };

        let expected = vec!["function Example.negate 0", "push argument 0", "neg", "return"];

        assert_eq!(expected, super::compile(class))
    }

    #[test]
    fn compile_if_statement() {
        /*
         * class Example {
         *     function boolean greater(int a, int b) {
         *         var boolean res;
         *
         *         if (a > b) {
         *             let res = true;
         *         } else {
         *             let res = false;
         *         }
         *
         *         return res;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Boolean),
                name: "greater".into(),
                parameters: vec![
                    Parameter { jack_type: JackType::Int, name: "a".into() },
                    Parameter { jack_type: JackType::Int, name: "b".into() },
                ],
                body: SubroutineBody {
                    vars: vec![VarDec { jack_type: JackType::Boolean, name: "res".into() }],
                    statements: vec![
                        Statement::If {
                            exp: Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![(Operation::LessThan, Term::VarName("b".into()))],
                            },
                            statements: vec![Statement::Let {
                                var_name: "res".into(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::KeywordConst("true".into()),
                                    additional: vec![],
                                },
                            }],
                            else_statements: Some(vec![Statement::Let {
                                var_name: "res".into(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::KeywordConst("false".into()),
                                    additional: vec![],
                                },
                            }]),
                        },
                        Statement::Return(Some(Expression {
                            term: Term::VarName("res".into()),
                            additional: vec![],
                        })),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.greater 1",
            "push argument 0",
            "push argument 1",
            "lt",
            "if-goto IfTrue$0",
            "goto IfFalse$0",
            "label IfTrue$0",
            "push constant 1",
            "neg",
            "pop local 0",
            "goto IfDone$0",
            "label IfFalse$0",
            "push constant 0",
            "pop local 0",
            "goto IfDone$0",
            "label IfDone$0",
            "push local 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_do_statement() {
        /*
         * class Example {
         *     function void test() {
         *         do Output.printInt(1);
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                name: "test".into(),
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printInt".into(),
                            expressions: vec![Expression {
                                term: Term::IntConst(1),
                                additional: vec![],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.test 0",
            "push constant 1",
            "call Output.printInt 1",
            "pop temp 0", /* gets rid of the return value since this is a do statement */
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_while_statement() {
        /*
         * class Example {
         *     function void incrUntilTen(int a) {
         *         while (a < 10) {
         *             let a = a + 1;
         *         }
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                name: "incrUntilTen".into(),
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                parameters: vec![Parameter { jack_type: JackType::Int, name: "a".into() }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![
                        Statement::While {
                            exp: Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![(Operation::LessThan, Term::IntConst(10))],
                            },
                            statements: vec![Statement::Let {
                                var_name: "a".into(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::VarName("a".into()),
                                    additional: vec![(Operation::Plus, Term::IntConst(1))],
                                },
                            }],
                        },
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.incrUntilTen 0",
            "label WhileCondition$0",
            "push argument 0",
            "push constant 10",
            "lt",
            "if-goto WhileStatements$0",
            "goto WhileDone$0",
            "label WhileStatements$0",
            "push argument 0",
            "push constant 1",
            "add",
            "pop argument 0",
            "goto WhileCondition$0",
            "label WhileDone$0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_function_calls() {
        /*
         * class Example {
         *     function void execute(int a) {
         *         let a = Example.incr(a);
         *         return;
         *     }
         *
         *     function int incr(int a) {
         *         let a = a + 1;
         *         return a;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![
                SubroutineDec {
                    routine_type: RoutineType::Function,
                    return_type: ReturnType::Void,
                    name: "execute".into(),
                    parameters: vec![Parameter { jack_type: JackType::Int, name: "a".into() }],
                    body: SubroutineBody {
                        vars: vec![],
                        statements: vec![
                            Statement::Let {
                                var_name: "a".into(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::Call(SubroutineCall {
                                        callee: Some("Example".into()),
                                        routine_name: "incr".into(),
                                        expressions: vec![Expression {
                                            term: Term::VarName("a".into()),
                                            additional: vec![],
                                        }],
                                    }),
                                    additional: vec![],
                                },
                            },
                            Statement::Return(None),
                        ],
                    },
                },
                SubroutineDec {
                    routine_type: RoutineType::Function,
                    return_type: ReturnType::Type(JackType::Int),
                    name: "incr".into(),
                    parameters: vec![Parameter { jack_type: JackType::Int, name: "a".into() }],
                    body: SubroutineBody {
                        vars: vec![],
                        statements: vec![
                            Statement::Let {
                                var_name: "a".into(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::VarName("a".into()),
                                    additional: vec![(Operation::Plus, Term::IntConst(1))],
                                },
                            },
                            Statement::Return(Some(Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![],
                            })),
                        ],
                    },
                },
            ],
        };

        let expected = vec![
            "function Example.execute 0",
            "push argument 0",
            "call Example.incr 1",
            "pop argument 0",
            "push constant 0",
            "return",
            "function Example.incr 0",
            "push argument 0",
            "push constant 1",
            "add",
            "pop argument 0",
            "push argument 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_constructor_caller() {
        /*
         * class Example {
         *     function createPoint() {
         *         var Point p1;
         *         let p1 = Point.new(1, 2);
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "createPoint".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![VarDec {
                        jack_type: JackType::Class("Point".into()),
                        name: "p1".into(),
                    }],
                    statements: vec![
                        Statement::Let {
                            var_name: "p1".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::Call(SubroutineCall {
                                    callee: Some("Point".into()),
                                    routine_name: "new".into(),
                                    expressions: vec![
                                        Expression { term: Term::IntConst(1), additional: vec![] },
                                        Expression { term: Term::IntConst(2), additional: vec![] },
                                    ],
                                }),
                                additional: vec![],
                            },
                        },
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.createPoint 1",
            "push constant 1",
            "push constant 2",
            "call Point.new 2",
            "pop local 0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_constructor_callee() {
        /*
         * class Point {
         *     field int x;
         *     field int y;
         *     constructor new(int argX, int argY) {
         *         let x = argX;
         *         let y = argY;
         *         return this;
         *     }
         * }
         */
        let class = Class {
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

        let expected = vec![
            "function Point.new 0",
            "push constant 2",
            "call Memory.alloc 1",
            "pop pointer 0",
            "push argument 0",
            "pop this 0",
            "push argument 1",
            "pop this 1",
            "push pointer 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_method_call_on_another_object() {
        /*
         * class Example {
         *     function int execute(Another arg) {
         *         return arg.execute(1);
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "execute".into(),
                parameters: vec![Parameter {
                    jack_type: JackType::Class("Another".into()),
                    name: "arg".into(),
                }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(Some(Expression {
                        term: Term::Call(SubroutineCall {
                            callee: Some("arg".into()),
                            routine_name: "execute".into(),
                            expressions: vec![Expression {
                                term: Term::IntConst(1),
                                additional: vec![],
                            }],
                        }),
                        additional: vec![],
                    }))],
                },
            }],
        };

        let expected = vec![
            "function Example.execute 0",
            "push argument 0",
            "push constant 1",
            "call Another.execute 2",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_output_print_string() {
        /*
         * class Main {
         *     function void main() {
         *         do Output.printString("Hello");
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Main".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "main".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printString".into(),
                            expressions: vec![Expression {
                                term: Term::StrConst("Hello".into()),
                                additional: vec![],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        /* Term::StrConst are initialized with String calls under the hood */
        let expected = vec![
            "function Main.main 0",
            "push constant 5",
            "call String.new 1",
            "push constant 72",
            "call String.appendChar 2",
            "push constant 101",
            "call String.appendChar 2",
            "push constant 108",
            "call String.appendChar 2",
            "push constant 108",
            "call String.appendChar 2",
            "push constant 111",
            "call String.appendChar 2",
            "call Output.printString 1",
            "pop temp 0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_method_and_method_call_on_current_object() {
        /*
         * class Example {
         *     field int x;
         *     contructor Example new(int argX) {
         *         let x = argX;
         *         return this;
         *     }
         *
         *     method int getIncrX() {
         *         return incrX();
         *     }
         *
         *     method int incrX() {
         *         let x = x + 1;
         *         return x;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![ClassVarDec {
                jack_type: JackType::Int,
                name: "x".into(),
                var_type: ClassVarType::Field,
            }],
            routines: vec![
                SubroutineDec {
                    routine_type: RoutineType::Constructor,
                    return_type: ReturnType::Type(JackType::Class("Example".into())),
                    name: "new".into(),
                    parameters: vec![Parameter { jack_type: JackType::Int, name: "argX".into() }],
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
                            Statement::Return(Some(Expression {
                                term: Term::KeywordConst("this".into()),
                                additional: vec![],
                            })),
                        ],
                    },
                },
                SubroutineDec {
                    routine_type: RoutineType::Method,
                    return_type: ReturnType::Type(JackType::Int),
                    name: "getIncrX".into(),
                    parameters: vec![],
                    body: SubroutineBody {
                        vars: vec![],
                        statements: vec![Statement::Return(Some(Expression {
                            term: Term::Call(SubroutineCall {
                                callee: None,
                                routine_name: "incrX".into(),
                                expressions: vec![],
                            }),
                            additional: vec![],
                        }))],
                    },
                },
                SubroutineDec {
                    routine_type: RoutineType::Method,
                    return_type: ReturnType::Type(JackType::Int),
                    name: "incrX".into(),
                    parameters: vec![],
                    body: SubroutineBody {
                        vars: vec![],
                        statements: vec![
                            Statement::Let {
                                var_name: "x".into(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::VarName("x".into()),
                                    additional: vec![(Operation::Plus, Term::IntConst(1))],
                                },
                            },
                            Statement::Return(Some(Expression {
                                term: Term::VarName("x".into()),
                                additional: vec![],
                            })),
                        ],
                    },
                },
            ],
        };

        let expected = vec![
            /* constructor */
            "function Example.new 0",
            "push constant 1",
            "call Memory.alloc 1",
            "pop pointer 0",
            "push argument 0",
            "pop this 0",
            "push pointer 0",
            "return",
            /* method int getIncrX */
            "function Example.getIncrX 0",
            "push argument 0",
            "pop pointer 0",
            "push pointer 0",
            "call Example.incrX 1",
            "return",
            /* method int incrX */
            "function Example.incrX 0",
            "push argument 0",
            "pop pointer 0",
            "push this 0",
            "push constant 1",
            "add",
            "pop this 0",
            "push this 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_method_call_as_do_statement() {
        /*
         * class Example {
         *     function void execute(Another arg) {
         *         do arg.execute(1);
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "execute".into(),
                parameters: vec![Parameter {
                    name: "arg".into(),
                    jack_type: JackType::Class("Another".into()),
                }],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![
                        Statement::Do(SubroutineCall {
                            callee: Some("arg".into()),
                            routine_name: "execute".into(),
                            expressions: vec![Expression {
                                term: Term::IntConst(1),
                                additional: vec![],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.execute 0",
            "push argument 0",
            "push constant 1",
            "call Another.execute 2",
            "pop temp 0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_array_init_and_access() {
        /* class Main {
         *     function void main() {
         *         var Array a;
         *         let a = Array.new(10);
         *         let a[0] = 19;
         *         do Output.printInt(a[0]);
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Main".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                name: "main".into(),
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![VarDec {
                        name: "a".into(),
                        jack_type: JackType::Class("Array".into()),
                    }],
                    statements: vec![
                        Statement::Let {
                            var_name: "a".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::Call(SubroutineCall {
                                    callee: Some("Array".into()),
                                    routine_name: "new".into(),
                                    expressions: vec![Expression {
                                        term: Term::IntConst(10),
                                        additional: vec![],
                                    }],
                                }),
                                additional: vec![],
                            },
                        },
                        Statement::Let {
                            var_name: "a".into(),
                            array_access: Some(Expression {
                                term: Term::IntConst(0),
                                additional: vec![],
                            }),
                            exp: Expression { term: Term::IntConst(19), additional: vec![] },
                        },
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printInt".into(),
                            expressions: vec![Expression {
                                term: Term::ArrayAccess {
                                    var_name: "a".into(),
                                    exp: Box::new(Expression {
                                        term: Term::IntConst(0),
                                        additional: vec![],
                                    }),
                                },
                                additional: vec![],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Main.main 1",
            "push constant 10",
            "call Array.new 1",
            "pop local 0",
            "push local 0",
            "push constant 0",
            "add",
            "push constant 19",
            "pop temp 0",
            "pop pointer 1",
            "push temp 0",
            "pop that 0",
            "push local 0",
            "push constant 0",
            "add",
            "pop pointer 1",
            "push that 0",
            "call Output.printInt 1",
            "pop temp 0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_string_var_assignment() {
        /*
         * class Example {
         *     function void execute() {
         *         var String s;
         *         let s = "test";
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "execute".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![VarDec {
                        name: "s".into(),
                        jack_type: JackType::Class("String".into()),
                    }],
                    statements: vec![
                        Statement::Let {
                            var_name: "s".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::StrConst("test".into()),
                                additional: vec![],
                            },
                        },
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Example.execute 1",
            "push constant 4",   /* len of 'test' */
            "call String.new 1", /* string constructor pushes on the stack the new string address */
            "push constant 116",
            "call String.appendChar 2",
            "push constant 101",
            "call String.appendChar 2",
            "push constant 115",
            "call String.appendChar 2",
            "push constant 116",
            "call String.appendChar 2",
            "pop local 0", /* assign address returned for the string to local var `s` */
            "push constant 0", /* return void */
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_nested_if_statements() {
        /*
         * class Main {
         *     function void main() {
         *         var int a;
         *         let a = 6;
         *         if (a < 10) {
         *             if (a > 5) {
         *                  let a = a + 1;
         *             } else {
         *                  let a = a + 2;
         *             }
         *         }
         *         do Output.printInt(a);
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Main".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                name: "main".into(),
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![VarDec { name: "a".into(), jack_type: JackType::Int }],
                    statements: vec![
                        Statement::Let {
                            var_name: "a".into(),
                            array_access: None,
                            exp: Expression { term: Term::IntConst(6), additional: vec![] },
                        },
                        Statement::If {
                            exp: Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![(Operation::LessThan, Term::IntConst(10))],
                            },
                            statements: vec![Statement::If {
                                exp: Expression {
                                    term: Term::VarName("a".into()),
                                    additional: vec![(Operation::GreaterThan, Term::IntConst(5))],
                                },
                                statements: vec![Statement::Let {
                                    var_name: "a".into(),
                                    array_access: None,
                                    exp: Expression {
                                        term: Term::VarName("a".into()),
                                        additional: vec![(Operation::Plus, Term::IntConst(1))],
                                    },
                                }],
                                else_statements: Some(vec![Statement::Let {
                                    var_name: "a".into(),
                                    array_access: None,
                                    exp: Expression {
                                        term: Term::VarName("a".into()),
                                        additional: vec![(Operation::Plus, Term::IntConst(2))],
                                    },
                                }]),
                            }],
                            else_statements: None,
                        },
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printInt".into(),
                            expressions: vec![Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Main.main 1",
            "push constant 6",
            "pop local 0",
            "push local 0",
            "push constant 10",
            "lt",
            "if-goto IfTrue$0",
            "goto IfFalse$0",
            "label IfTrue$0",
            "push local 0",
            "push constant 5",
            "gt",
            "if-goto IfTrue$1",
            "goto IfFalse$1",
            "label IfTrue$1",
            "push local 0",
            "push constant 1",
            "add",
            "pop local 0",
            "goto IfDone$1",
            "label IfFalse$1",
            "push local 0",
            "push constant 2",
            "add",
            "pop local 0",
            "goto IfDone$1",
            "label IfDone$1",
            "goto IfDone$0",
            "label IfFalse$0",
            "goto IfDone$0",
            "label IfDone$0",
            "push local 0",
            "call Output.printInt 1",
            "pop temp 0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class));
    }

    #[test]
    fn compile_unary_operation() {
        /*
         * class Main {
         *     function void main() {
         *         var int a;
         *         let a = 5;
         *         let a = ~a;
         *         do Output.printInt(a);
         *         return;
         *     }
         * }
         */
        let class = Class {
            name: "Main".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                name: "main".into(),
                return_type: ReturnType::Void,
                parameters: vec![],
                routine_type: RoutineType::Function,
                body: SubroutineBody {
                    vars: vec![VarDec { name: "a".into(), jack_type: JackType::Int }],
                    statements: vec![
                        Statement::Let {
                            var_name: "a".into(),
                            array_access: None,
                            exp: Expression { term: Term::IntConst(5), additional: vec![] },
                        },
                        Statement::Let {
                            var_name: "a".into(),
                            array_access: None,
                            exp: Expression {
                                term: Term::Unary {
                                    op: Operation::Not,
                                    term: Box::new(Term::VarName("a".into())),
                                },
                                additional: vec![],
                            },
                        },
                        Statement::Do(SubroutineCall {
                            callee: Some("Output".into()),
                            routine_name: "printInt".into(),
                            expressions: vec![Expression {
                                term: Term::VarName("a".into()),
                                additional: vec![],
                            }],
                        }),
                        Statement::Return(None),
                    ],
                },
            }],
        };

        let expected = vec![
            "function Main.main 1",
            "push constant 5",
            "pop local 0",
            "push local 0",
            "not",
            "pop local 0",
            "push local 0",
            "call Output.printInt 1",
            "pop temp 0",
            "push constant 0",
            "return",
        ];

        assert_eq!(expected, super::compile(class))
    }
}
