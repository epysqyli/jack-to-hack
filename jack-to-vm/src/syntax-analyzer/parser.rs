use crate::syntax_analyzer::tokenizer::Token;

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
///   <className>
///     <identifier> Main </identifier>
///   </className>
///   <symbol> { </symbol>
///   <subroutineDec>
///     <keyword> function </keyword>
///     <keyword> void </keyword>
///     <subroutineName>
///       <identifier> main </identifier>
///     </subroutineName>
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
#[allow(dead_code)]
pub fn parse(tokens: Vec<Token>) {
    let mut index = 0;
    if let Token::Keyword(val) = &tokens[index] {
        if val.as_str() == "class" {
            eval_class(&tokens, &mut index);
        }
    }
}

/* ===================================== */
/* ========= Program Structure ========= */
/* ===================================== */

/* 'class' className '{' classVarDec* subroutineDec* '}' */
fn eval_class(tokens: &Vec<Token>, index: &mut usize) {
    println!("<class>");
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_class_name(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    while let Token::Keyword(val) = &tokens[*index] {
        if !["static", "field", "constructor", "function", "method"].contains(&val.as_str()) {
            break;
        }

        match val.as_str() {
            "static" | "field" => eval_class_var_dec(tokens, index),
            "constructor" | "function" | "method" => eval_subroutine_dec(tokens, index),
            _ => {}
        }

        if *index == &tokens.len() - 1 {
            break;
        }

        *index += 1;
    }

    println!("{}", &tokens[*index]);
    println!("</class>");
}

/* ('static'|'field') type varName (',' varName)* ';' */
fn eval_class_var_dec(tokens: &Vec<Token>, index: &mut usize) {
    println!("<classVarDec>");
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_type(tokens, index);
    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    while let Token::Symbol(val) = &tokens[*index] {
        if val == ";" {
            println!("{}", &tokens[*index]);
            break;
        }

        println!("{}", &tokens[*index]);
        if let Token::Symbol(_) = &tokens[*index] {
            *index += 1;
            eval_var_name(tokens, index);
            *index += 1;
        }
    }

    println!("</classVarDec>");
}

/* ('constructor'|'function'|'method') ('void'|type) subroutineName '(' parameterList ')' subroutineBody */
fn eval_subroutine_dec(tokens: &Vec<Token>, index: &mut usize) {
    println!("<subroutineDec>");
    println!("{}", &tokens[*index]);

    *index += 1;
    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "void" => println!("{}", &tokens[*index]),
            "int" | "char" | "boolean" => eval_type(tokens, index),
            _ => { /* no rule */ }
        },
        Token::Identifier(_) => eval_type(tokens, index),
        _ => { /* rule error */ }
    }

    *index += 1;
    if let Token::Identifier(_) = &tokens[*index] {
        eval_subroutine_name(tokens, index);
    }

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    match &tokens[*index] {
        Token::Symbol(val) => println!("{}", &tokens[*index]),
        _ => {
            eval_parameter_list(tokens, index);
            *index += 1;
            println!("{}", &tokens[*index]);
        }
    }

    *index += 1;
    println!("{}", &tokens[*index]);

    if let Token::Keyword(val) = &tokens[*index + 1] {
        if ["var", "let", "if", "do", "while", "return"].contains(&val.as_str()) {
            eval_subroutine_body(tokens, index);
        }
    }

    *index += 1;
    println!("{}", &tokens[*index]);
    println!("</subroutineDec>");
}

/* 'int'|'char'|'boolean'|className */
fn eval_type(tokens: &Vec<Token>, index: &mut usize) {
    println!("<type>");

    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "int" | "char" | "boolean" => println!("{}", &tokens[*index]),
            _ => { /* no rule */ }
        },
        Token::Identifier(_) => eval_class_name(tokens, index),
        _ => { /* rule error */ }
    }

    println!("</type>");
}

/* identifier */
fn eval_subroutine_name(tokens: &Vec<Token>, index: &mut usize) {
    println!("<subroutineName>");
    println!("{}", &tokens[*index]);
    println!("</subroutineName>");
}

/* ( (type varName) (',' type varName)* )? */
fn eval_parameter_list(tokens: &Vec<Token>, index: &mut usize) {
    println!("<parameterList>");

    eval_type(tokens, index);
    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    while let Token::Symbol(val) = &tokens[*index] {
        println!("{}", &tokens[*index]);
        *index += 1;
        eval_type(tokens, index);
        *index += 1;
        eval_var_name(tokens, index);
    }

    println!("</parameterList>");
}

/* identifier */
fn eval_var_name(tokens: &Vec<Token>, index: &mut usize) {
    println!("<varName>");
    println!("{}", &tokens[*index]);
    println!("</varName>");
}

/* identifier */
fn eval_class_name(tokens: &Vec<Token>, index: &mut usize) {
    println!("<className>");
    println!("{}", &tokens[*index]);
    println!("</className>");
}

/* '{' varDec* statements '}' */
fn eval_subroutine_body(tokens: &Vec<Token>, index: &mut usize) {
    println!("<subroutineBody>");

    *index += 1;
    while let Token::Keyword(val) = &tokens[*index] {
        match val.as_str() {
            "var" => eval_var_dec(tokens, index),
            "let" | "if" | "do" | "while" | "return" => eval_statements(tokens, index),
            _ => {}
        }

        *index += 1;
    }

    println!("</subroutineBody>");
}

/* 'var' type varName (',' varName)* ';' */
fn eval_var_dec(tokens: &Vec<Token>, index: &mut usize) {
    println!("<varDec>");
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_type(tokens, index);
    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    while let Token::Symbol(val) = &tokens[*index] {
        if val == ";" {
            println!("{}", &tokens[*index]);
            break;
        }

        println!("{}", &tokens[*index]);
        if let Token::Symbol(_) = &tokens[*index] {
            *index += 1;
            eval_var_name(tokens, index);
            *index += 1;
        }
    }

    println!("</varDec>");
}

/* ============================== */
/* ========= Statements ========= */
/* ============================== */

/* statement* */
fn eval_statements(tokens: &Vec<Token>, index: &mut usize) {
    println!("<statements>");

    if let Token::Keyword(val) = &tokens[*index] {
        if ["if", "let", "do", "while", "return"].contains(&val.as_str()) {
            eval_statement(tokens, index);
        }
    }

    println!("</statements>");
}

/* letStatement|ifStatement|whileStatement|doStatement|returnStatement */
fn eval_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<statement>");

    if let Token::Keyword(val) = &tokens[*index] {
        match val.as_str() {
            "return" => eval_return_statement(tokens, index),
            "if" => eval_if_statement(tokens, index),
            "let" => eval_let_statement(tokens, index),
            "do" => eval_do_statement(tokens, index),
            "while" => eval_while_statement(tokens, index),
            _ => { /* no rule */ }
        }
    }

    println!("</statement>");
}

/* 'return' expression? ';' */
fn eval_return_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<returnStatement>");
    println!("{}", &tokens[*index]);

    *index += 1;
    match &tokens[*index] {
        Token::Symbol(_) => println!("{}", &tokens[*index]),
        _ => eval_expression(tokens, index),
    }

    println!("</returnStatement>");
}

/* 'if' '(' expression ')' '{' statements '}' ( 'else' '{' statements '}' )? */
fn eval_if_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<ifStatement>");

    println!("{}", &tokens[*index]);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_expression(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_statements(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    if let Token::Keyword(val) = &tokens[*index + 1] {
        if val.as_str() == "else" {
            *index += 1;
            println!("{}", &tokens[*index]);
            *index += 1;
            println!("{}", &tokens[*index]);
            *index += 1;
            eval_statements(tokens, index);
            *index += 1;
            println!("{}", &tokens[*index]);
        }
    }

    println!("</ifStatement>");
}

/* 'let' varName ('[' expression ']')? '=' expression ';' */
fn eval_let_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<letStatement>");
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    if &tokens[*index] == &Token::Symbol("[".to_string()) {
        *index += 1;
        eval_expression(tokens, index);
        println!("{}", &tokens[*index]);
        *index += 1;
    }

    *index += 1;
    eval_expression(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    println!("</letStatement>");
}

/* 'while' '(' expression ')' '{' statements '}' */
fn eval_while_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<whileStatement>");

    println!("{}", &tokens[*index]);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_expression(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_statements(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    println!("</whileStatement>");
}

/* 'do' subroutineCall ';' */
fn eval_do_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<doStatement>");
    
    println!("{}", &tokens[*index]);

    *index += 1;
    eval_subroutine_call(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    println!("</doStatement>");
}

/* =============================== */
/* ========= Expressions ========= */
/* =============================== */

/* term (op term)* */
fn eval_expression(tokens: &Vec<Token>, index: &mut usize) {
    println!("<expression>");

    eval_term(tokens, index);

    while let Token::Symbol(val) = &tokens[*index + 1] {
        if ["+", "-", "*", "/", "&", "|", "<", ">", "="].contains(&val.as_str()) {
            *index += 1;
            eval_op(tokens, index);
        } else {
            break;
        }

        *index += 1;
        eval_term(tokens, index);
    }

    println!("</expression>");
}

/*
 * integerConstant | stringConstant | keywordConstant | varName
 * | varName '[' expression ']' | '(' expression ')' | (unaryOp term)
 * | subroutineCall
 */
fn eval_term(tokens: &Vec<Token>, index: &mut usize) {
    println!("<term>");

    match &tokens[*index] {
        Token::IntConst(val) => println!("{}", &tokens[*index]),
        Token::StrConst(val) => println!("{}>", &tokens[*index]),
        Token::Keyword(_) => eval_keyword_constant(tokens, index),
        /* '(' expression ')' | (unaryOp term) */
        Token::Symbol(val) => match val.as_str() {
            "-" | "~" => {
                eval_unary_op(tokens, index);
                *index += 1;
                eval_term(tokens, index);
            }
            "(" => {
                println!("{}", &tokens[*index]);
                *index += 1;
                eval_expression(tokens, index);
                *index += 1;
                println!("{}", &tokens[*index]);
            }
            _ => { /* no rule */ }
        },
        Token::Identifier(_) => {
            /* varName | varName '[' expression ']' | subroutineCall */
            match &tokens[*index + 1] {
                Token::Symbol(val) => match val.as_str() {
                    "(" | "." => eval_subroutine_call(tokens, index),
                    "[" => {
                        eval_var_name(tokens, index);
                        *index += 1;
                        println!("{}", &tokens[*index]);
                        *index += 1;
                        eval_expression(tokens, index);
                        *index += 1;
                        println!("{}", &tokens[*index]);
                    }
                    _ => {
                        eval_var_name(tokens, index);
                    }
                },
                _ => { /* no rule */ }
            }
        }
    }

    println!("</term>");
}

/* subroutineName '(' expressionList ')' |
 * (className | varName) '.' subroutineName '(' expressionList ')'
 */
fn eval_subroutine_call(tokens: &Vec<Token>, index: &mut usize) {
    println!("<subroutineCall>");

    if let Token::Symbol(val) = &tokens[*index + 1] {
        if val.as_str() == "." {
            if let Token::Identifier(val) = &tokens[*index] {
                if val.chars().nth(0).unwrap().is_ascii_uppercase() {
                    eval_class_name(tokens, index);
                } else {
                    eval_var_name(tokens, index);
                }
            }

            *index += 1;
            println!("{}", &tokens[*index]);
            *index += 1;
        }
    }

    println!("{}", &tokens[*index]);

    *index += 1;
    println!("{}", &tokens[*index]);

    eval_expression_list(tokens, index);

    *index += 1;
    println!("{}", &tokens[*index]);

    println!("</subroutineCall>");
}

/* '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=' */
fn eval_op(tokens: &Vec<Token>, index: &mut usize) {
    println!("<op>");
    println!("{}", &tokens[*index]);
    println!("</op>");
}

/* '-' | '~' */
fn eval_unary_op(tokens: &Vec<Token>, index: &mut usize) {
    println!("<unaryOp>");
    println!("{}", &tokens[*index]);
    println!("</unaryOp>");
}

/* 'true' | 'false' | 'null' | 'this' */
fn eval_keyword_constant(tokens: &Vec<Token>, index: &mut usize) {
    println!("<keywordConst>");
    println!("{}", &tokens[*index]);
    println!("</keywordConst>");
}

/* (expression (',' expression)* )? */
fn eval_expression_list(tokens: &Vec<Token>, index: &mut usize) {
    println!("<expressionList>");

    if let Token::Symbol(val) = &tokens[*index + 1] {
        if val.as_str() == ")" {
            println!("</expressionList>");
            return;
        }
    }

    *index += 1;
    eval_expression(tokens, index);

    while let Token::Symbol(val) = &tokens[*index] {
        if val.as_str() == ")" {
            break;
        }

        if val.as_str() == "," {
            println!("{}", &tokens[*index]);
            *index += 1;
            eval_expression(tokens, index);
        }
    }

    println!("</expressionList>");
}

#[cfg(test)]
mod tests {
    use super::{
        Token::{self, *},
        parse,
    };

    #[test]
    fn parse_test_class() {
        let token_stream: Vec<Token> = vec![
            //
            // class Main {
            Keyword("class".to_string()),
            Identifier("Main".to_string()),
            Symbol("{".to_string()),
            //
            // function int test() {
            Keyword("function".to_string()),
            Keyword("void".to_string()),
            Identifier("Main".to_string()),
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
            Symbol("}".to_string()),
            Symbol(";".to_string()),
            //
            // return ; }
            Keyword("return".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
            //
            // }
            Symbol("}".to_string()),
        ];

        println!();
        parse(token_stream);
        println!();
        assert!(true);
    }
}
