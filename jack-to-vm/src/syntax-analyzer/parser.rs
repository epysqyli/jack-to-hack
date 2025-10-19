use crate::syntax_analyzer::tokenizer::Token;
use std::fmt::Write;

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
///   <keyword> class </keyword>
///   <className> Main </className>
///   <symbol> { </symbol>
///   <subroutineDec>
///     <keyword> function </keyword>
///     <keyword> void </keyword>
///     <subroutineName> main </subroutineName>
///     <symbol> ( </symbol>
///     <parameterList></parameterList>
///     <symbol> ) </symbol>
///     <subroutineBody>
///       <symbol> { </symbol>
///       <statements>
///           <statement>
///             <returnStatement>
///                 <keyword> return </keyword>
///                 <symbol> ; </symbol>
///             </returnStatement>
///           </statement>
///       </statements>
///       <symbol> } </symbol>
///     </subroutineBody>
///   </subroutineDec>
///   <symbol> } </symbol>
/// </class>

pub struct Parser {
    index: usize,
    tokens: Vec<Token>,
    output: String,
}

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> String {
        let mut parser = Self {
            index: 0,
            tokens: tokens,
            output: String::new(),
        };

        parser.eval_class();
        parser.output
    }

    /* ================================= */
    /* ======= Program Structure ======= */
    /* ================================= */

    /* 'class' className '{' classVarDec* subroutineDec* '}' */
    fn eval_class(self: &mut Self) {
        self.append(Some("<class>"));
        self.append(None);

        self.advance();
        self.eval_class_name();

        self.advance();
        self.append(None);

        self.advance();

        while let Token::Keyword(val) = self.current() {
            if !["static", "field", "constructor", "function", "method"].contains(&val.as_str()) {
                break;
            }

            match val.as_str() {
                "static" | "field" => self.eval_class_var_dec(),
                "constructor" | "function" | "method" => self.eval_subroutine_dec(),
                _ => {}
            }

            if self.index == self.tokens.len() - 1 {
                break;
            }

            self.advance();
        }

        self.append(None);
        self.append(Some("</class>"));
    }

    /* ('static'|'field') type varName (',' varName)* ';' */
    fn eval_class_var_dec(self: &mut Self) {
        self.append(Some("<classVarDec>"));
        self.append(None);

        self.advance();
        self.eval_type();

        self.advance();
        self.eval_var_name();

        self.advance();

        while let Token::Symbol(val) = self.current() {
            if val == ";" {
                self.append(None);
                break;
            }

            self.append(None);

            if let Token::Symbol(_) = self.current() {
                self.advance();
                self.eval_var_name();
                self.advance();
            }
        }

        self.append(Some("</classVarDec>"));
    }

    /* ('constructor'|'function'|'method') ('void'|type) subroutineName '(' parameterList ')' subroutineBody */
    fn eval_subroutine_dec(self: &mut Self) {
        self.append(Some("<subroutineDec>"));
        self.append(None);

        self.advance();
        match self.current() {
            Token::Keyword(val) => match val.as_str() {
                "void" => self.append(None),
                "int" | "char" | "boolean" => self.eval_type(),
                _ => { /* no rule */ }
            },
            Token::Identifier(_) => self.eval_type(),
            _ => { /* rule error */ }
        }

        self.advance();
        if let Token::Identifier(_) = self.current() {
            self.eval_subroutine_name();
        }

        self.advance();
        self.append(None);

        self.advance();

        match self.current() {
            Token::Symbol(_) => self.append(None),
            _ => {
                self.eval_parameter_list();
                self.advance();
                self.append(None);
            }
        }

        self.advance();
        self.append(None);

        if let Token::Keyword(val) = self.next() {
            if ["var", "let", "if", "do", "while", "return"].contains(&val.as_str()) {
                self.eval_subroutine_body();
            }
        }

        self.advance();
        self.append(None);
        self.append(Some("</subroutineDec>"));
    }

    /* 'int'|'char'|'boolean'|className */
    fn eval_type(self: &mut Self) {
        self.append(Some("<type>"));

        match self.current() {
            Token::Keyword(val) => match val.as_str() {
                "int" | "char" | "boolean" => self.append(None),
                _ => { /* no rule */ }
            },
            Token::Identifier(_) => self.eval_class_name(),
            _ => { /* no rule */ }
        }

        self.append(Some("</type>"));
    }

    /* identifier */
    fn eval_subroutine_name(self: &mut Self) {
        self.append(Some("<subroutineName>"));
        self.append(None);
        self.append(Some("</subroutineName>"));
    }

    /* ( (type varName) (',' type varName)* )? */
    fn eval_parameter_list(self: &mut Self) {
        self.append(Some("<parameterList>"));

        self.eval_type();

        self.advance();
        self.eval_var_name();

        self.advance();

        while let Token::Symbol(_) = self.current() {
            self.append(None);
            self.advance();
            self.eval_type();
            self.advance();
            self.eval_var_name();
        }

        self.append(Some("</parameterList>"));
    }

    /* identifier */
    fn eval_var_name(self: &mut Self) {
        self.append(Some("<varName>"));
        self.append(None);
        self.append(Some("</varName>"));
    }

    /* identifier */
    fn eval_class_name(self: &mut Self) {
        self.append(Some("<className>"));
        self.append(None);
        self.append(Some("</className>"));
    }

    /* '{' varDec* statements '}' */
    fn eval_subroutine_body(self: &mut Self) {
        self.append(Some("<subroutineBody>"));

        self.advance();

        while let Token::Keyword(val) = self.current() {
            match val.as_str() {
                "var" => self.eval_var_dec(),
                "let" | "if" | "do" | "while" | "return" => self.eval_statements(),
                _ => {}
            }

            self.advance();
        }

        self.append(Some("</subroutineBody>"));
    }

    /* 'var' type varName (',' varName)* ';' */
    fn eval_var_dec(self: &mut Self) {
        self.append(Some("<varDec>"));
        self.append(None);

        self.advance();
        self.eval_type();

        self.advance();
        self.eval_var_name();

        self.advance();

        while let Token::Symbol(val) = self.current() {
            if val == ";" {
                self.append(None);
                break;
            }

            self.append(None);
            if let Token::Symbol(_) = self.current() {
                self.advance();
                self.eval_var_name();
                self.advance();
            }
        }

        self.append(Some("</varDec>"));
    }

    /* ============================== */
    /* ========= Statements ========= */
    /* ============================== */

    /* statement* */
    fn eval_statements(self: &mut Self) {
        self.append(Some("<statements>"));

        if let Token::Keyword(val) = self.current() {
            if ["if", "let", "do", "while", "return"].contains(&val.as_str()) {
                self.eval_statement();
            }
        }

        self.append(Some("</statements>"));
    }

    /* letStatement|ifStatement|whileStatement|doStatement|returnStatement */
    fn eval_statement(self: &mut Self) {
        self.append(Some("<statement>"));

        if let Token::Keyword(val) = self.current() {
            match val.as_str() {
                "return" => self.eval_return_statement(),
                "if" => self.eval_if_statement(),
                "let" => self.eval_let_statement(),
                "do" => self.eval_do_statement(),
                "while" => self.eval_while_statement(),
                _ => { /* no rule */ }
            }
        }

        self.append(Some("</statement>"));
    }

    /* 'return' expression? ';' */
    fn eval_return_statement(self: &mut Self) {
        self.append(Some("<returnStatement>"));
        self.append(None);

        self.advance();
        match self.current() {
            Token::Symbol(_) => self.append(None),
            _ => self.eval_expression(),
        }

        self.append(Some("</returnStatement>"));
    }

    /* 'if' '(' expression ')' '{' statements '}' ( 'else' '{' statements '}' )? */
    fn eval_if_statement(self: &mut Self) {
        self.append(Some("<ifStatement>"));

        self.append(None);

        self.advance();
        self.append(None);

        self.advance();
        self.eval_expression();

        self.advance();
        self.append(None);

        self.advance();
        self.append(None);

        self.advance();
        self.eval_statements();

        self.advance();
        self.append(None);

        if let Token::Keyword(val) = self.next() {
            if val.as_str() == "else" {
                self.advance();
                self.append(None);

                self.advance();
                self.append(None);

                self.advance();
                self.eval_statements();

                self.advance();
                self.append(None);
            }
        }

        self.append(Some("</ifStatement>"));
    }

    /* 'let' varName ('[' expression ']')? '=' expression ';' */
    fn eval_let_statement(self: &mut Self) {
        self.append(Some("<letStatement>"));
        self.append(None);

        self.advance();
        self.eval_var_name();

        self.advance();
        self.append(None);

        if self.current() == &Token::Symbol("[".to_string()) {
            self.advance();
            self.eval_expression();
            self.append(None);
            self.advance();
        }

        self.advance();
        self.eval_expression();

        self.advance();
        self.append(None);

        self.append(Some("</letStatement>"));
    }

    /* 'while' '(' expression ')' '{' statements '}' */
    fn eval_while_statement(self: &mut Self) {
        self.append(Some("<whileStatement>"));

        self.append(None);

        self.advance();
        self.append(None);

        self.advance();
        self.eval_expression();

        self.advance();
        self.append(None);

        self.advance();
        self.append(None);

        self.advance();
        self.eval_statements();

        self.advance();
        self.append(None);

        self.append(Some("</whileStatement>"));
    }

    /* 'do' subroutineCall ';' */
    fn eval_do_statement(self: &mut Self) {
        self.append(Some("<doStatement>"));

        self.append(None);

        self.advance();
        self.eval_subroutine_call();

        self.advance();
        self.append(None);

        self.append(Some("</doStatement>"));
    }

    /* =============================== */
    /* ========= Expressions ========= */
    /* =============================== */

    /* term (op term)* */
    fn eval_expression(self: &mut Self) {
        self.append(Some("<expression>"));
        self.eval_term();

        while let Token::Symbol(val) = self.next() {
            if ["+", "-", "*", "/", "&", "|", "<", ">", "="].contains(&val.as_str()) {
                self.advance();
                self.eval_op();
            } else {
                break;
            }

            self.advance();
            self.eval_term();
        }

        self.append(Some("</expression>"));
    }

    /*
     * integerConstant | stringConstant | keywordConstant | varName
     * | varName '[' expression ']' | '(' expression ')' | (unaryOp term)
     * | subroutineCall
     */
    fn eval_term(self: &mut Self) {
        self.append(Some("<term>"));

        match self.current() {
            Token::IntConst(_) | Token::StrConst(_) => self.append(None),
            Token::Keyword(_) => self.eval_keyword_constant(),
            /* '(' expression ')' | (unaryOp term) */
            Token::Symbol(val) => match val.as_str() {
                "-" | "~" => {
                    self.eval_unary_op();
                    self.advance();
                    self.eval_term();
                }
                "(" => {
                    self.append(None);
                    self.advance();
                    self.eval_expression();
                    self.advance();
                    self.append(None);
                }
                _ => { /* no rule */ }
            },
            Token::Identifier(_) => {
                /* varName | varName '[' expression ']' | subroutineCall */
                match self.next() {
                    Token::Symbol(val) => match val.as_str() {
                        "(" | "." => self.eval_subroutine_call(),
                        "[" => {
                            self.eval_var_name();

                            self.advance();
                            self.append(None);

                            self.advance();
                            self.eval_expression();

                            self.advance();
                            self.append(None);
                        }
                        _ => {
                            self.eval_var_name();
                        }
                    },
                    _ => { /* no rule */ }
                }
            }
        }

        self.append(Some("</term>"));
    }

    /* subroutineName '(' expressionList ')' |
     * (className | varName) '.' subroutineName '(' expressionList ')'
     */
    fn eval_subroutine_call(self: &mut Self) {
        self.append(Some("<subroutineCall>"));

        if let Token::Symbol(val) = self.next() {
            if val.as_str() == "." {
                if let Token::Identifier(val) = self.current() {
                    if val.chars().nth(0).unwrap().is_ascii_uppercase() {
                        self.eval_class_name();
                    } else {
                        self.eval_var_name();
                    }
                }

                self.advance();
                self.append(None);

                self.advance();
            }
        }

        self.append(None);

        self.advance();
        self.append(None);

        self.eval_expression_list();

        self.advance();
        self.append(None);

        self.append(Some("</subroutineCall>"));
    }

    /* '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=' */
    fn eval_op(self: &mut Self) {
        self.append(Some("<op>"));
        self.append(None);
        self.append(Some("</op>"));
    }

    /* '-' | '~' */
    fn eval_unary_op(self: &mut Self) {
        self.append(Some("<unaryOp>"));
        self.append(None);
        self.append(Some("</unaryOp>"));
    }

    /* 'true' | 'false' | 'null' | 'this' */
    fn eval_keyword_constant(self: &mut Self) {
        self.append(Some("<keywordConst>"));
        self.append(None);
        self.append(Some("</keywordConst>"));
    }

    /* (expression (',' expression)* )? */
    fn eval_expression_list(self: &mut Self) {
        self.append(Some("<expressionList>"));

        if let Token::Symbol(val) = self.next() {
            if val.as_str() == ")" {
                self.append(Some("</expressionList>"));
                return;
            }
        }

        self.next();
        self.eval_expression();

        while let Token::Symbol(val) = self.current() {
            if val.as_str() == ")" {
                break;
            }

            if val.as_str() == "," {
                self.append(None);
                self.next();
                self.eval_expression();
            }
        }

        self.append(Some("</expressionList>"));
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

    fn append(self: &mut Self, val: Option<&str>) {
        match val {
            None => write!(self.output, "{}", self.tokens[self.index]).unwrap(),
            Some(v) => write!(self.output, "{v}").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Token::{self, *};

    #[test]
    fn parse_while_statement() {
        let token_stream: Vec<Token> = vec![
            //
            // class Main {
            Keyword("class".to_string()),
            Identifier("Main".to_string()),
            Symbol("{".to_string()),
            //
            // function void main() {
            Keyword("function".to_string()),
            Keyword("void".to_string()),
            Identifier("main".to_string()),
            Symbol("(".to_string()),
            Symbol(")".to_string()),
            Symbol("{".to_string()),
            //
            // while (count < 100) { let count = count + 1; }
            Keyword("while".to_string()),
            Symbol("(".to_string()),
            Identifier("count".to_string()),
            Symbol("<".to_string()),
            IntConst("100".to_string()),
            Symbol(")".to_string()),
            Symbol("{".to_string()),
            Keyword("let".to_string()),
            Identifier("count".to_string()),
            Symbol("=".to_string()),
            Identifier("count".to_string()),
            Symbol("+".to_string()),
            IntConst("1".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
            //
            // return ; }
            Keyword("return".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
            //
            // }
            Symbol("}".to_string()),
        ];

        let expected = r#"
            <class>
                <keyword>class</keyword>
                <className>Main</className>
                <symbol>{</symbol>
                <subroutineDec>
                    <keyword>function</keyword>
                    <keyword>void</keyword>
                    <subroutineName>main</subroutineName>
                    <symbol>(</symbol>
                    <symbol>)</symbol>
                    <symbol>{</symbol>
                    <subroutineBody>
                        <statements>
                            <statement>
                                <whileStatement>
                                    <keyword>while</keyword>
                                    <symbol>(</symbol>
                                    <expression>
                                        <term><varName>count</varName></term>
                                        <op><symbol><</symbol></op>
                                        <term><intConst>100</intConst></term>
                                    </expression>
                                    <symbol>)</symbol>
                                    <symbol>{</symbol>
                                    <statements>
                                        <statement>
                                            <letStatement>
                                                <keyword>let</keyword>
                                                <varName>count</varName>
                                                <symbol>=</symbol>
                                                <expression>
                                                    <term><varName>count</varName></term>
                                                    <op><symbol>+</symbol></op>
                                                    <term><intConst>1</intConst></term>
                                                </expression>
                                                <symbol>;</symbol>
                                            </letStatement>
                                        </statement>
                                    </statements>
                                    <symbol>}</symbol>
                                </whileStatement>
                            </statement>
                        </statements>
                        <statements>
                            <statement>
                                <returnStatement>
                                    <keyword>return</keyword>
                                    <symbol>;</symbol>
                                </returnStatement>
                            </statement>
                        </statements>
                    </subroutineBody>
                    <symbol>}</symbol>
                </subroutineDec>
                <symbol>}</symbol>
            </class>
        "#;

        assert_eq!(
            expected.replace(" ", "").replace("\n", ""),
            super::Parser::parse(token_stream)
        );
    }
}
