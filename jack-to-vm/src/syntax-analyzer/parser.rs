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
    // TODO:
    // - make sure qualifiers are evaluated properly for each rule: * ? |
    // - make sure the index is advanced correctly and in a consistent way across rules
    // - think about in-memory derivation tree type only when the grammar logic is implemented
    //   - print to stdout for the moment
    let mut index = 0;
    if let Token::Keyword(val) = &tokens[index] {
        if val.as_str() == "class" {
            eval_class(&tokens, &mut index);
        }
    }
}

/* 'class' className '{' classVarDec* subroutineDec* '}' */
fn eval_class(tokens: &Vec<Token>, index: &mut usize) {
    println!("<class>");
    println!("<keyword>{}</keyword>", &tokens[*index]);

    *index += 1;
    eval_class_name(tokens, index);

    *index += 1;
    println!("<symbol>{}</symbol>", &tokens[*index]);

    *index += 1;
    eval_class_var_dec_and_subroutine_dec(tokens, index);

    println!("<symbol>{}</symbol>", &tokens[*index]);
    println!("</class>");
}

/* classVarDec and subroutineDec can occur multiple times within a classRule */
fn eval_class_var_dec_and_subroutine_dec(tokens: &Vec<Token>, index: &mut usize) {
    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "static" | "field" => eval_class_var_dec(tokens, index),
            "constructor" | "function" | "method" => eval_subroutine_dec(tokens, index),
            _ => {}
        },
        _ => {}
    }

    if *index + 1 == tokens.len() {
        return
    }

    *index += 1;
    if let Token::Keyword(val) = &tokens[*index] {
        if ["static", "field", "constructor", "function", "method"].contains(&val.as_str()) {
            eval_class_var_dec_and_subroutine_dec(tokens, index);
        }
    }
}

/* ('static'|'field') type varName (',' varName)* ';' */
fn eval_class_var_dec(tokens: &Vec<Token>, index: &mut usize) {
    println!("<classVarDec>");
    println!("<keyword>{}</keyword>", &tokens[*index]);

    *index += 1;
    eval_type(tokens, index);
    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    while let Token::Symbol(val) = &tokens[*index] {
        if val == ";" {
            println!("<symbol>{}</symbol>", val);
            break;
        }

        println!("<symbol>{}</symbol>", val);
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
    println!("<keyword>{}</keyword>", &tokens[*index]);

    *index += 1;
    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "void" => println!("<keyword>{}</keyword>", val),
            "int" | "char" | "boolean" => eval_type(tokens, index),
            _ => { /* no rule */ }
        },
        Token::Identifier(_) => eval_type(tokens, index),
        _ => { /* rule error */ }
    }

    *index += 1;
    match &tokens[*index] {
        Token::Identifier(_) => eval_subroutine_name(tokens, index),
        _ => { /* rule error */ }
    }

    *index += 1;
    println!("<symbol>{}</symbol>", &tokens[*index]);

    *index += 1;
    match &tokens[*index] {
        Token::Symbol(val) => println!("<symbol>{}</symbol>", val),
        _ => {
            eval_parameter_list(tokens, index);
            *index += 1;
            println!("<symbol>{}</symbol>", &tokens[*index]);
        }
    }

    *index += 1;
    println!("<symbol>{}</symbol>", &tokens[*index]);

    *index += 1;
    eval_subroutine_body(tokens, index);

    *index += 1;
    println!("<symbol>{}</symbol>", &tokens[*index]);
    println!("</subroutineDec>");
}

/* 'int'|'char'|'boolean'|className */
fn eval_type(tokens: &Vec<Token>, index: &mut usize) {
    println!("<type>");

    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "int" | "char" | "boolean" => println!("<keyword>{}</keyword>", val),
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
    println!("<identifier>{}</identifier>", &tokens[*index]);
    println!("</subroutineName>");
}

/* ( (type varName) (',' type varName)* )? */
fn eval_parameter_list(tokens: &Vec<Token>, index: &mut usize) {
    println!("<parameterList>");

    eval_type(tokens, index);
    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    match &tokens[*index] {
        Token::Symbol(val) => match val.as_str() {
            "," => eval_parameter_list(tokens, index),
            _ => { /* no rule */ }
        },
        _ => { /* no rule */ }
    }

    println!("</parameterList>");
}

/* identifier */
fn eval_var_name(tokens: &Vec<Token>, index: &mut usize) {
    println!("<varName>");
    println!("<identifier>{}</identifier>", &tokens[*index]);
    println!("</varName>");
}

/* identifier */
fn eval_class_name(tokens: &Vec<Token>, index: &mut usize) {
    println!("<className>");
    println!("<identifier>{}</identifier>", &tokens[*index]);
    println!("</className>");
}

/* '{' varDec* statements '}' */
fn eval_subroutine_body(tokens: &Vec<Token>, index: &mut usize) {
    println!("<subroutineBody>");
    eval_var_dec_and_statements(tokens, index);
    println!("</subroutineBody>");
}

/* varDec and statements can occur multiple times within a subroutineBody */
fn eval_var_dec_and_statements(tokens: &Vec<Token>, index: &mut usize) {
    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "var" => eval_var_dec(tokens, index),
            "let" | "if" | "do" | "while" | "return" => eval_statements(tokens, index),
            _ => {}
        },
        _ => {}
    }

    *index += 1;
    if let Token::Keyword(val) = &tokens[*index] {
        if ["var", "let", "if", "do", "while", "return"].contains(&val.as_str()) {
            eval_var_dec_and_statements(tokens, index);
        }
    }
}

/* 'var' type varName (',' varName)* ';' */
fn eval_var_dec(tokens: &Vec<Token>, index: &mut usize) {
    println!("<varDec>");
    println!("<keyword>{}</keyword>", &tokens[*index]);

    *index += 1;
    eval_type(tokens, index);
    *index += 1;
    eval_var_name(tokens, index);

    *index += 1;
    while let Token::Symbol(val) = &tokens[*index] {
        if val == ";" {
            println!("<symbol>{}</symbol>", val);
            break;
        }

        println!("<symbol>{}</symbol>", val);
        if let Token::Symbol(_) = &tokens[*index] {
            *index += 1;
            eval_var_name(tokens, index);
            *index += 1;
        }
    }

    println!("</varDec>");
}

/* statement* */
fn eval_statements(tokens: &Vec<Token>, index: &mut usize) {
    println!("<statements>");

    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "if" | "let" | "do" | "while" | "return" => eval_statement(tokens, index),
            _ => { /* no rule */ }
        },
        _ => { /* rule error */ }
    }

    println!("</statements>");
}

/* letStatement|ifStatement|whileStatement|doStatement|returnStatement */
fn eval_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<statement>");

    match &tokens[*index] {
        Token::Keyword(val) => match val.as_str() {
            "return" => eval_return_statement(tokens, index),
            "if" => {}
            "let" => {}
            "do" => {}
            "while" => {}
            _ => { /* no rule */ }
        },
        _ => { /* rule error */ }
    }

    println!("</statement>");
}

/* 'return' expression? ';' */
fn eval_return_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<returnStatement>");
    println!("<keyword>{}</keyword>", &tokens[*index]);

    *index += 1;
    match &tokens[*index] {
        Token::Symbol(_) => println!("<symbol>{}</symbol>", &tokens[*index]),
        _ => { /* implement eval_expression */ }
    }

    println!("</returnStatement>");
}

/* 'if' '(' expression ')' '{' statements '}' */
fn eval_if_statement(tokens: &Vec<Token>, index: &mut usize) {
    println!("<ifStatement>");
    // TODO: implement
    println!("</ifStatement>");
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
            // class Test {
            Keyword("class".to_string()),
            Identifier("Test".to_string()),
            Symbol("{".to_string()),
            //
            // function void test() {
            Keyword("function".to_string()),
            Keyword("void".to_string()),
            Identifier("test".to_string()),
            Symbol("(".to_string()),
            Symbol(")".to_string()),
            Symbol("{".to_string()),
            //
            // var int localVar;
            Keyword("var".to_string()),
            Keyword("int".to_string()),
            Identifier("localVar".to_string()),
            Symbol(";".to_string()),
            //
            // var char anotherLocalVar;
            Keyword("var".to_string()),
            Keyword("char".to_string()),
            Identifier("anotherLocalVar".to_string()),
            Symbol(";".to_string()),
            //
            // return; }
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
