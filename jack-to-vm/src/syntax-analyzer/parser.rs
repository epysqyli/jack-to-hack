use super::super::grammar::*;
use super::tokenizer::Token;

/// Tokens -> recursive application of grammar rules -> derivation tree
///
/// The parse tree has one starting node, the `class` rule/node, and an arbitrary number
/// of recursively nested nodes which are the result of further rule evaluations.
/// Each rule evaluation results in either a terminal or non-terminal node, and
/// stops when all non-terminal nodes are evaluated into terminal ones.
///
/// The class is the unit of compilation.
/// Each .jack file declares exactly one class.
///
/// The following Jack class ...
///
/// class Main {
///      function void main() {
///          return;
///      }
/// }
///
/// ... should evaluate to the following derivation tree:
///
/// <class>
///     <keyword>class</keyword>
///     <identifier>Main</identifier>
///     <symbol>{</symbol>
///     <subroutineDec>
///         <keyword>function</keyword>
///         <keyword>void</keyword>
///         <identifier>main</identifier>
///         <symbol>(</symbol>
///         <symbol>)</symbol>
///         <symbol>{</symbol>
///             <subroutineBody>
///                 <statements>
///                     <returnStatement>
///                         <keyword>return</keyword>
///                         <symbol>;</symbol>
///                     </returnStatement>
///                 </statements>
///             </subroutineBody>
///         <symbol>}</symbol>
///     </subroutineDec>
///     <symbol>}</symbol>
/// </class>
///
/// The XML representation is simply a human readable one.
pub fn parse(tokens: Vec<Token>) -> Class {
    let mut parser = Parser::new(tokens);
    parser.eval_class()
}

struct Parser {
    index: usize,
    tokens: Vec<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { index: 0, tokens }
    }

    /* ================================= */
    /* ======= Program Structure ======= */
    /* ================================= */

    /* 'class' className '{' classVarDec* subroutineDec* '}' */
    fn eval_class(self: &mut Self) -> Class {
        self.advance();
        let class_name = self.eval_class_name();
        self.advance();
        self.advance();

        let mut class_var_decs: Vec<ClassVarDec> = vec![];
        let mut subroutine_decs: Vec<SubroutineDec> = vec![];

        while let Token::Keyword(val) = self.current() {
            if !["static", "field", "constructor", "function", "method"].contains(&val.as_str()) {
                break;
            }

            match val.as_str() {
                "static" | "field" => {
                    self.eval_class_var_dec()
                        .into_iter()
                        .for_each(|c| class_var_decs.push(c));
                }
                "constructor" | "function" | "method" => {
                    subroutine_decs.push(self.eval_subroutine_dec());
                }
                _ => {}
            }

            if self.index == self.tokens.len() - 1 {
                break;
            }

            self.advance();
        }

        Class {
            name: class_name,
            vars: class_var_decs,
            routines: subroutine_decs,
        }
    }

    /* ('static'|'field') type varName (',' varName)* ';' */
    fn eval_class_var_dec(self: &mut Self) -> Vec<ClassVarDec> {
        let mut class_var_decs: Vec<ClassVarDec> = vec![];
        let var_type = format!("{}", self.tokens[self.index]);

        self.advance();
        let jack_type = self.eval_type();

        self.advance();
        let var_name = self.eval_var_name();

        class_var_decs.push(ClassVarDec {
            var_type: var_type.try_into().unwrap(),
            jack_type: jack_type.try_into().unwrap(),
            name: var_name,
        });

        self.advance();

        while let Token::Symbol(val) = self.current() {
            if val == ";" {
                break;
            }

            if let Token::Symbol(_) = self.current() {
                self.advance();

                class_var_decs.push(ClassVarDec {
                    var_type: class_var_decs[0].var_type.clone(),
                    jack_type: class_var_decs[0].jack_type.clone(),
                    name: self.eval_var_name(),
                });

                self.advance();
            }
        }

        class_var_decs
    }

    /* ('constructor'|'function'|'method') ('void'|type) subroutineName '(' parameterList ')' subroutineBody */
    fn eval_subroutine_dec(self: &mut Self) -> SubroutineDec {
        let routine_type = format!("{}", self.current());
        self.advance();

        let return_type = match self.current() {
            Token::Keyword(val) => match val.as_str() {
                "void" => val.to_owned(),
                "int" | "char" | "boolean" => self.eval_type(),
                _ => "".to_string(),
            },
            Token::Identifier(_) => self.eval_type(),
            _ => "".to_string(),
        };

        self.advance();
        let routine_name = if let Token::Identifier(_) = self.current() {
            self.eval_subroutine_name()
        } else {
            "".to_string() // panic, actually
        };

        self.advance();
        self.advance();
        let mut parameters: Vec<Parameter> = vec![];

        match self.current() {
            Token::Symbol(_) => { /* do nothing */ }
            _ => {
                parameters = self.eval_parameter_list();
            }
        }

        self.advance();
        let mut routine_body: SubroutineBody = SubroutineBody::default();

        if let Token::Keyword(val) = self.next() {
            if ["var", "let", "if", "do", "while", "return"].contains(&val.as_str()) {
                self.advance();
                routine_body = self.eval_subroutine_body();
            }
        }

        self.advance();

        SubroutineDec {
            routine_type: routine_type.try_into().unwrap(),
            return_type: return_type.try_into().unwrap(),
            name: routine_name,
            parameters: parameters,
            body: routine_body,
        }
    }

    /* 'int'|'char'|'boolean'|className */
    fn eval_type(self: &mut Self) -> String {
        match self.current() {
            Token::Keyword(val) => match val.as_str() {
                "int" | "char" | "boolean" => format!("{}", self.tokens[self.index]),
                _ => panic!("Should have been int|char|boolean"),
            },
            Token::Identifier(_) => self.eval_class_name(),
            _ => panic!("Should have been className"),
        }
    }

    /* identifier */
    fn eval_subroutine_name(self: &mut Self) -> String {
        format!("{}", self.current())
    }

    /* ( (type varName) (',' type varName)* )? */
    fn eval_parameter_list(self: &mut Self) -> Vec<Parameter> {
        let mut parameters: Vec<Parameter> = vec![];

        let param_type = self.eval_type();

        self.advance();
        let param_name = self.eval_var_name();

        parameters.push(Parameter {
            jack_type: param_type.try_into().unwrap(),
            name: param_name,
        });

        self.advance();

        while let Token::Symbol(_) = self.current() {
            if self.current() == &Token::Symbol(")".into()) {
                break;
            }

            self.advance();
            let param_type = self.eval_type();
            self.advance();
            let param_name = self.eval_var_name();
            parameters.push(Parameter {
                jack_type: param_type.try_into().unwrap(),
                name: param_name,
            });
        }

        parameters
    }

    /* identifier */
    fn eval_var_name(self: &mut Self) -> String {
        format!("{}", self.tokens[self.index])
    }

    /* identifier */
    fn eval_class_name(self: &mut Self) -> String {
        format!("{}", self.tokens[self.index])
    }

    /* '{' varDec* statements '}' */
    fn eval_subroutine_body(self: &mut Self) -> SubroutineBody {
        let mut vars: Vec<VarDec> = vec![];
        let mut statements: Vec<Statement> = vec![];

        while let Token::Keyword(val) = self.current() {
            match val.as_str() {
                "var" => {
                    self.eval_var_dec().into_iter().for_each(|v| vars.push(v));
                }
                "let" | "if" | "do" | "while" | "return" => {
                    self.eval_statements()
                        .into_iter()
                        .for_each(|s| statements.push(s));
                }
                _ => {}
            }

            self.advance();
        }

        SubroutineBody { vars, statements }
    }

    /* 'var' type varName (',' varName)* ';' */
    fn eval_var_dec(self: &mut Self) -> Vec<VarDec> {
        let mut var_decs: Vec<VarDec> = vec![];

        self.advance();
        let var_type = self.eval_type();

        self.advance();
        let var_name = self.eval_var_name();

        var_decs.push(VarDec {
            jack_type: var_type.try_into().unwrap(),
            name: var_name,
        });

        self.advance();

        while let Token::Symbol(val) = self.current() {
            if val == ";" {
                break;
            }

            if let Token::Symbol(_) = self.current() {
                self.advance();
                let var_name = self.eval_var_name();

                var_decs.push(VarDec {
                    jack_type: var_decs[0].jack_type.to_owned(),
                    name: var_name,
                });

                self.advance();
            }
        }

        var_decs
    }

    /* ============================== */
    /* ========= Statements ========= */
    /* ============================== */

    /* statement* */
    fn eval_statements(self: &mut Self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];

        if let Token::Keyword(val) = self.current() {
            if ["if", "let", "do", "while", "return"].contains(&val.as_str()) {
                statements.push(self.eval_statement());
            }
        }

        statements
    }

    /* letStatement|ifStatement|whileStatement|doStatement|returnStatement */
    fn eval_statement(self: &mut Self) -> Statement {
        match self.current() {
            Token::Keyword(val) => match val.as_str() {
                "return" => self.eval_return_statement(),
                "if" => self.eval_if_statement(),
                "let" => self.eval_let_statement(),
                "do" => self.eval_do_statement(),
                "while" => self.eval_while_statement(),
                _ => panic!("Should have been a Statement"),
            },
            _ => panic!("Should have been a Statement"),
        }
    }

    /* 'return' expression? ';' */
    fn eval_return_statement(self: &mut Self) -> Statement {
        self.advance();
        match self.current() {
            Token::Symbol(val) => {
                if val.as_str() == "-" || val.as_str() == "~" {
                    Statement::Return(Some(self.eval_expression()))
                } else {
                    Statement::Return(None)
                }
            }
            _ => Statement::Return(Some(self.eval_expression())),
        }
    }

    /* 'if' '(' expression ')' '{' statements '}' ( 'else' '{' statements '}' )? */
    fn eval_if_statement(self: &mut Self) -> Statement {
        self.advance();
        self.advance();
        let exp = self.eval_expression();

        self.advance();
        self.advance();
        self.advance();
        let statements = self.eval_statements();

        self.advance();

        let else_statements = if let Token::Keyword(val) = self.next()
            && val.as_str() == "else"
        {
            self.advance();
            self.advance();
            self.advance();
            let else_statements = self.eval_statements();
            self.advance();
            Some(else_statements)
        } else {
            None
        };

        Statement::If {
            exp,
            statements,
            else_statements,
        }
    }

    /* 'let' varName ('[' expression ']')? '=' expression ';' */
    fn eval_let_statement(self: &mut Self) -> Statement {
        self.advance();
        let var_name = self.eval_var_name();

        self.advance();

        let array_access = if self.current() == &Token::Symbol("[".to_string()) {
            self.advance();
            let exp = self.eval_expression();
            self.advance();
            Some(exp)
        } else {
            None
        };

        self.advance();
        let exp = self.eval_expression();
        self.advance();

        Statement::Let {
            var_name,
            array_access,
            exp,
        }
    }

    /* 'while' '(' expression ')' '{' statements '}' */
    fn eval_while_statement(self: &mut Self) -> Statement {
        self.advance();
        self.advance();
        let exp = self.eval_expression();

        self.advance();
        self.advance();
        self.advance();
        let statements = self.eval_statements();

        self.advance();
        Statement::While { exp, statements }
    }

    /* 'do' subroutineCall ';' */
    fn eval_do_statement(self: &mut Self) -> Statement {
        self.advance();
        let subroutine_call = self.eval_subroutine_call();
        self.advance();

        if let Term::Call(call) = subroutine_call {
            Statement::Do(call)
        } else {
            panic!("Should return a Statement")
        }
    }

    /* =============================== */
    /* ========= Expressions ========= */
    /* =============================== */

    /* term (op term)* */
    fn eval_expression(self: &mut Self) -> Expression {
        let term = self.eval_term();
        let mut additional: Vec<(Operation, Term)> = vec![];

        while let Token::Symbol(val) = self.next() {
            if ["+", "-", "*", "/", "&", "|", "<", ">", "="].contains(&val.as_str()) {
                self.advance();
                let op = self.eval_op();
                self.advance();
                let op_term = self.eval_term();
                additional.push((op, op_term));
            } else {
                break;
            }
        }

        Expression { term, additional }
    }

    /*
     * integerConstant | stringConstant | keywordConstant | varName
     * | varName '[' expression ']' | '(' expression ')' | (unaryOp term)
     * | subroutineCall
     */
    fn eval_term(self: &mut Self) -> Term {
        match self.current() {
            Token::IntConst(val) => Term::IntConst(val.parse().unwrap()),
            Token::StrConst(val) => Term::StrConst(val.to_owned()),
            Token::Keyword(_) => self.eval_keyword_constant(),
            /* '(' expression ')' | (unaryOp term) */
            Token::Symbol(val) => match val.as_str() {
                "(" => {
                    self.advance();
                    let exp = self.eval_expression();
                    self.advance();
                    Term::Expression(Box::new(exp))
                }
                "-" | "~" => {
                    let unary_op = self.eval_unary_op();
                    self.advance();
                    let term = self.eval_term();
                    Term::Unary {
                        op: unary_op,
                        term: Box::new(term),
                    }
                }
                _ => panic!("Not a term"),
            },
            Token::Identifier(_) => {
                /* varName | varName '[' expression ']' | subroutineCall */
                match self.next() {
                    Token::Symbol(val) => match val.as_str() {
                        "(" | "." => self.eval_subroutine_call(),
                        "[" => {
                            let var_name = self.eval_var_name();
                            self.advance();

                            self.advance();
                            let exp = self.eval_expression();
                            self.advance();

                            Term::ArrayAccess {
                                var_name: var_name,
                                exp: exp.into(),
                            }
                        }
                        _ => Term::VarName(self.eval_var_name()),
                    },
                    _ => panic!("Not a term"),
                }
            }
        }
    }

    /* subroutineName '(' expressionList ')' |
     * (className | varName) '.' subroutineName '(' expressionList ')' */
    fn eval_subroutine_call(self: &mut Self) -> Term {
        let callee = if let Token::Symbol(val) = self.next()
            && val.as_str() == "."
            && let Token::Identifier(val) = self.current()
        {
            let name = if val.chars().nth(0).unwrap().is_ascii_uppercase() {
                self.eval_class_name()
            } else {
                self.eval_var_name()
            };

            self.advance();
            self.advance();
            Some(name)
        } else {
            None
        };

        let routine_name = format!("{}", self.current());

        self.advance();

        let exps = if let Token::Symbol(val) = self.next()
            && val.as_str() == ")"
        {
            vec![]
        } else {
            self.eval_expression_list()
        };

        self.advance();

        Term::Call(SubroutineCall {
            callee: callee,
            routine_name: routine_name,
            expressions: exps,
        })
    }

    /* '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=' */
    fn eval_op(self: &mut Self) -> Operation {
        Operation::try_from(format!("{}", self.current())).unwrap()
    }

    /* '-' | '~' */
    fn eval_unary_op(self: &mut Self) -> Operation {
        Operation::try_from(format!("{}", self.current())).unwrap()
    }

    /* 'true' | 'false' | 'null' | 'this' */
    fn eval_keyword_constant(self: &mut Self) -> Term {
        Term::KeywordConst(format!("{}", self.current()))
    }

    /* (expression (',' expression)* )? */
    fn eval_expression_list(self: &mut Self) -> Vec<Expression> {
        let mut exps: Vec<Expression> = vec![];

        self.next();
        exps.push(self.eval_expression());

        while let Token::Symbol(val) = self.current() {
            if val.as_str() == ")" {
                break;
            }

            if val.as_str() == "," {
                self.advance();
                exps.push(self.eval_expression());
            }
        }

        exps
    }

    /* ================================== */
    /* ========= Helper Methods ========= */
    /* ================================== */

    fn current(self: &Self) -> &Token {
        &self.tokens[self.index]
    }

    fn next(self: &Self) -> &Token {
        &self.tokens[self.index + 1]
    }

    fn advance(self: &mut Self) {
        self.index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::grammar::*;
    use super::Token::*;

    #[test]
    fn parse_class() {
        let token_stream = vec![
            //
            // class Example {
            Keyword("class".to_string()),
            Identifier("Example".to_string()),
            Symbol("{".to_string()),
            //
            // field int a, b;
            Keyword("field".to_string()),
            Keyword("int".to_string()),
            Identifier("a".to_string()),
            Symbol(",".to_string()),
            Identifier("b".to_string()),
            Symbol(";".to_string()),
            //
            // function int example () {
            Keyword("function".to_string()),
            Keyword("int".to_string()),
            Identifier("example".to_string()),
            Symbol("(".to_string()),
            Symbol(")".to_string()),
            Symbol("{".to_string()),
            //
            // while (a < 10)
            Keyword("while".to_string()),
            Symbol("(".to_string()),
            Identifier("a".to_string()),
            Symbol("<".to_string()),
            IntConst("10".to_string()),
            Symbol(")".to_string()),
            //
            // { let a = a + 1; }
            Symbol("{".to_string()),
            Keyword("let".to_string()),
            Identifier("a".to_string()),
            Symbol("=".to_string()),
            Identifier("a".to_string()),
            Symbol("+".to_string()),
            IntConst("1".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
            //
            // return a; }
            Keyword("return".to_string()),
            Identifier("a".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
            //
            // }
            Symbol("}".to_string()),
        ];

        let expected = Class {
            name: "Example".to_owned(),
            vars: vec![
                ClassVarDec {
                    var_type: ClassVarType::Field,
                    jack_type: JackType::Int,
                    name: "a".to_owned(),
                },
                ClassVarDec {
                    var_type: ClassVarType::Field,
                    jack_type: JackType::Int,
                    name: "b".to_owned(),
                },
            ],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Type(JackType::Int),
                name: "example".to_owned(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![
                        Statement::While {
                            exp: Expression {
                                term: Term::VarName("a".to_owned()),
                                additional: vec![(Operation::LessThan, Term::IntConst(10))],
                            },
                            statements: vec![Statement::Let {
                                var_name: "a".to_owned(),
                                array_access: None,
                                exp: Expression {
                                    term: Term::VarName("a".to_owned()),
                                    additional: vec![(Operation::Plus, Term::IntConst(1))],
                                },
                            }],
                        },
                        Statement::Return(Some(Expression {
                            term: Term::VarName("a".to_owned()),
                            additional: vec![],
                        })),
                    ],
                },
            }],
        };

        assert_eq!(expected, super::parse(token_stream));
    }

    #[test]
    fn parse_empty_class() {
        let tokens = vec![
            Keyword("class".into()),
            Identifier("Main".into()),
            Symbol("{".into()),
            Symbol("}".into()),
        ];

        let expected = Class {
            name: "Main".into(),
            vars: vec![],
            routines: vec![],
        };

        assert_eq!(expected, super::Parser::new(tokens).eval_class());
    }

    #[test]
    fn parse_class_var_declarations() {
        let tokens = vec![
            Keyword("static".into()),
            Keyword("int".into()),
            Identifier("a".into()),
            Symbol(",".into()),
            Identifier("b".into()),
            Symbol(";".into()),
        ];

        let expected = vec![
            ClassVarDec {
                var_type: ClassVarType::Static,
                jack_type: JackType::Int,
                name: "a".into(),
            },
            ClassVarDec {
                var_type: ClassVarType::Static,
                jack_type: JackType::Int,
                name: "b".into(),
            },
        ];

        assert_eq!(expected, super::Parser::new(tokens).eval_class_var_dec());
    }

    #[test]
    fn parse_subroutine_declaration() {
        let tokens = vec![
            Keyword("function".into()),
            Keyword("int".into()),
            Identifier("incr".into()),
            Symbol("(".into()),
            Keyword("int".into()),
            Identifier("a".into()),
            Symbol(")".into()),
            Symbol("{".into()),
            Keyword("return".into()),
            Identifier("a".into()),
            Symbol("+".into()),
            IntConst("1".into()),
            Symbol(";".into()),
            Symbol("}".into()),
        ];

        let expected = SubroutineDec {
            routine_type: RoutineType::Function,
            return_type: ReturnType::Type(JackType::Int),
            name: "incr".into(),
            parameters: vec![Parameter {
                jack_type: JackType::Int,
                name: "a".into(),
            }],
            body: SubroutineBody {
                vars: vec![],
                statements: vec![Statement::Return(Some(Expression {
                    term: Term::VarName("a".into()),
                    additional: vec![(Operation::Plus, Term::IntConst(1))],
                }))],
            },
        };

        assert_eq!(expected, super::Parser::new(tokens).eval_subroutine_dec());
    }

    #[test]
    fn parse_subroutine_body() {
        let tokens: Vec<super::Token> = vec![
            Keyword("var".into()),
            Keyword("int".into()),
            Identifier("localA".into()),
            Symbol(";".into()),
            Keyword("var".into()),
            Keyword("boolean".into()),
            Identifier("localB".into()),
            Symbol(";".into()),
            Keyword("let".into()),
            Identifier("localA".into()),
            Symbol("=".into()),
            IntConst("1".into()),
            Symbol(";".into()),
            Symbol("}".into()), /* only required due to `eval_subroutine_body` implementation */
        ];

        let expected = SubroutineBody {
            vars: vec![
                VarDec {
                    jack_type: JackType::Int,
                    name: "localA".into(),
                },
                VarDec {
                    jack_type: JackType::Boolean,
                    name: "localB".into(),
                },
            ],
            statements: vec![Statement::Let {
                var_name: "localA".into(),
                array_access: None,
                exp: Expression {
                    term: Term::IntConst(1),
                    additional: vec![],
                },
            }],
        };

        assert_eq!(expected, super::Parser::new(tokens).eval_subroutine_body());
    }

    #[test]
    fn parse_let_statement() {
        let tokens = vec![
            Keyword("let".into()),
            Identifier("a".into()),
            Symbol("=".into()),
            Identifier("a".into()),
            Symbol("+".into()),
            IntConst("1".into()),
            Symbol(";".into()),
        ];

        let expected = Statement::Let {
            var_name: "a".into(),
            array_access: None,
            exp: Expression {
                term: Term::VarName("a".into()),
                additional: vec![(Operation::Plus, Term::IntConst(1))],
            },
        };

        let actual = super::Parser::new(tokens).eval_let_statement();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_return_statement() {
        let tokens = vec![
            Keyword("return".into()),
            Identifier("a".into()),
            Symbol(";".into()),
        ];

        let expected = Statement::Return(Some(Expression {
            term: Term::VarName("a".into()),
            additional: vec![],
        }));

        let actual = super::Parser::new(tokens).eval_return_statement();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_if_statement() {
        let tokens = vec![
            Keyword("if".into()),
            Symbol("(".into()),
            Identifier("a".into()),
            Symbol("<".into()),
            IntConst("5".into()),
            Symbol(")".into()),
            Symbol("{".into()),
            Keyword("let".into()),
            Identifier("a".into()),
            Symbol("=".into()),
            Identifier("a".into()),
            Symbol("+".into()),
            IntConst("1".into()),
            Symbol(";".into()),
            Symbol("}".into()),
            Keyword("else".into()),
            Symbol("{".into()),
            Keyword("let".into()),
            Identifier("a".into()),
            Symbol("=".into()),
            Identifier("a".into()),
            Symbol("*".into()),
            IntConst("2".into()),
            Symbol("}".into()),
        ];

        let expected = Statement::If {
            exp: Expression {
                term: Term::VarName("a".into()),
                additional: vec![(Operation::LessThan, Term::IntConst(5))],
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
                    additional: vec![(Operation::Multiply, Term::IntConst(2))],
                },
            }]),
        };

        let actual = super::Parser::new(tokens).eval_if_statement();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_while_statement() {
        let tokens = vec![
            Keyword("while".into()),
            Symbol("(".into()),
            Identifier("a".into()),
            Symbol("<".into()),
            IntConst("5".into()),
            Symbol(")".into()),
            Symbol("{".into()),
            Keyword("let".into()),
            Identifier("a".into()),
            Symbol("=".into()),
            Identifier("a".into()),
            Symbol("+".into()),
            IntConst("1".into()),
            Symbol(";".into()),
            Symbol("}".into()),
        ];

        let expected = Statement::While {
            exp: Expression {
                term: Term::VarName("a".into()),
                additional: vec![(Operation::LessThan, Term::IntConst(5))],
            },
            statements: vec![Statement::Let {
                var_name: "a".into(),
                array_access: None,
                exp: Expression {
                    term: Term::VarName("a".into()),
                    additional: vec![(Operation::Plus, Term::IntConst(1))],
                },
            }],
        };

        let actual = super::Parser::new(tokens).eval_while_statement();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_do_statement() {
        let tokens = vec![
            Keyword("do".into()),
            Identifier("AnotherClass".into()),
            Symbol(".".into()),
            Identifier("incr".into()),
            Symbol("(".into()),
            Identifier("a".into()),
            Symbol(")".into()),
            Symbol(";".into()),
            Symbol(";".into()),
        ];

        let expected = Statement::Do(SubroutineCall {
            callee: Some("AnotherClass".into()),
            routine_name: "incr".into(),
            expressions: vec![Expression {
                term: Term::Expression(Box::new(Expression {
                    term: Term::VarName("a".into()),
                    additional: vec![],
                })),
                additional: vec![],
            }],
        });

        assert_eq!(expected, super::Parser::new(tokens).eval_do_statement());
    }
}
