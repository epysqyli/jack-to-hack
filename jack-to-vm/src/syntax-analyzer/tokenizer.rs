use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Symbol(String),
    Identifier(String),
    StrConst(String),
    IntConst(String),
}

impl Display for Token {
    fn fmt(self: &Self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyword(val) => write!(f, "<keyword>{val}</keyword>"),
            Self::Symbol(val) => write!(f, "<symbol>{val}</symbol>"),
            Self::Identifier(val) => write!(f, "<identifier>{val}</identifier>"),
            Self::StrConst(val) => write!(f, "<strConst>{val}</strConst>"),
            Self::IntConst(val) => write!(f, "<intConst>{val}</intConst>"),
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let input_without_comments = remove_comments(input);
    let candidates = identify_candidate_tokens(input_without_comments);

    candidates
        .iter()
        .map(|candidate| {
            if candidate.starts_with('\"') && candidate.ends_with('\"') {
                return Token::StrConst(candidate.trim_matches('\"').to_string());
            }

            if lexicon::KEYWORDS.contains(&candidate.as_str()) {
                return Token::Keyword(candidate.to_string());
            }

            if lexicon::SYMBOLS.contains(&candidate.chars().collect::<Vec<char>>()[0]) {
                return Token::Symbol(candidate.to_string());
            }

            match candidate.parse::<usize>() {
                Ok(_) => Token::IntConst(candidate.to_string()),
                Err(_) => Token::Identifier(candidate.to_string()),
            }
        })
        .collect()
}

fn remove_comments(input: &str) -> String {
    input
        .lines()
        .map(|l| {
            if !l.contains("/*") && !l.contains("//") {
                return l.to_string();
            }

            let mut clean_line: String = String::new();
            let mut index: usize = 0;
            let chars: Vec<char> = l.chars().collect();
            let len = chars.len();
            let mut within_comment: bool = false;

            while index < len - 1 {
                if [&chars[index], &chars[index + 1]] == [&'/', &'/'] {
                    break;
                }

                if [&chars[index], &chars[index + 1]] == [&'/', &'*'] {
                    within_comment = true;
                    index += 2;
                    continue;
                }

                if index < len - 2
                    && [&chars[index], &chars[index + 1], &chars[index + 2]] == [&'/', &'*', &'*']
                {
                    within_comment = true;
                    index += 3;
                    continue;
                }

                if [&chars[index], &chars[index + 1]] == [&'*', &'/'] {
                    within_comment = false;
                    index += 2;
                    continue;
                }

                if within_comment {
                    index += 1;
                    continue;
                }

                clean_line.push(chars[index]);
                index += 1;

                if index + 1 == len {
                    clean_line.push(chars[index]);
                    break;
                }
            }

            clean_line
        })
        .map(|l| l.trim().trim_matches('\t').trim_matches('\n').to_string())
        .collect()
}

fn identify_candidate_tokens(input_without_comments: String) -> Vec<String> {
    let mut index: usize = 0;
    let mut within_string_literal = false;
    let chars: Vec<char> = input_without_comments.chars().collect();
    let mut candidate: Vec<char> = vec![];
    let mut candidates: Vec<String> = vec![];

    while index < chars.len() {
        if chars[index] == '\"' {
            within_string_literal = !within_string_literal;
            candidate.push(chars[index]);
            index += 1;
            continue;
        }

        if within_string_literal {
            candidate.push(chars[index]);
            index += 1;
            continue;
        }

        if lexicon::SYMBOLS.contains(&chars[index]) {
            if !candidate.is_empty() {
                candidates.push(candidate.iter().collect());
                candidate = vec![];
            }
            candidates.push(chars[index].to_string());
            index += 1;
            continue;
        }

        if chars[index] == ' ' {
            if !candidate.is_empty() {
                candidates.push(candidate.iter().collect());
                candidate = vec![];
            }
            index += 1;
            continue;
        }

        candidate.push(chars[index]);
        index += 1;
    }

    candidates
}

mod lexicon {
    pub const KEYWORDS: [&str; 21] = [
        "class",
        "constructor",
        "function",
        "method",
        "field",
        "static",
        "var",
        "int",
        "char",
        "boolean",
        "void",
        "true",
        "false",
        "null",
        "this",
        "let",
        "do",
        "if",
        "else",
        "while",
        "return",
    ];

    pub const SYMBOLS: [char; 19] = [
        '{', '}', '[', ']', '(', ')', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=',
        '~',
    ];
}

#[cfg(test)]
mod tests {
    use super::{
        Token::{self, *},
        tokenize,
    };

    fn simple_source_expected_tokens() -> Vec<Token> {
        vec![
            Keyword("if".to_string()),
            Symbol("(".to_string()),
            Identifier("x".to_string()),
            Symbol("<".to_string()),
            IntConst("0".to_string()),
            Symbol(")".to_string()),
            Symbol("{".to_string()),
            Keyword("let".to_string()),
            Identifier("sign".to_string()),
            Symbol("=".to_string()),
            StrConst("negative".to_string()),
            Symbol(";".to_string()),
            Keyword("let".to_string()),
            Identifier("anotherSign".to_string()),
            Symbol("=".to_string()),
            StrConst("positive".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
        ]
    }

    #[test]
    fn tokenize_simple_source() {
        let source = r#"if (x < 0) { let sign = "negative"; let anotherSign = "positive"; }"#;
        assert_eq!(simple_source_expected_tokens(), tokenize(source))
    }

    #[test]
    fn tokenize_simple_multiline_source() {
        let source = r#"if (x < 0) {
			let sign = "negative";
            let anotherSign = "positive";
		}"#;

        assert_eq!(simple_source_expected_tokens(), tokenize(source))
    }

    #[test]
    fn tokenize_simple_source_with_comments() {
        let source = r#"if (x < 0) {
			let sign = "negative"; // handles the sign
            /** docblock */
            let anotherSign = /* another comment */ "positive";
		}"#;

        assert_eq!(simple_source_expected_tokens(), tokenize(source))
    }

    #[test]
    fn tokenize_main_class_example() {
        let expected: Vec<Token> = vec![
            Keyword("class".to_string()),
            Identifier("Main".to_string()),
            Symbol("{".to_string()),
            Keyword("function".to_string()),
            Keyword("void".to_string()),
            Identifier("main".to_string()),
            Symbol("(".to_string()),
            Symbol(")".to_string()),
            Symbol("{".to_string()),
            Keyword("do".to_string()),
            Identifier("Output".to_string()),
            Symbol(".".to_string()),
            Identifier("printString".to_string()),
            Symbol("(".to_string()),
            StrConst("Hello World !".to_string()),
            Symbol(")".to_string()),
            Symbol(";".to_string()),
            Keyword("do".to_string()),
            Identifier("Output".to_string()),
            Symbol(".".to_string()),
            Identifier("println".to_string()),
            Symbol("(".to_string()),
            Symbol(")".to_string()),
            Symbol(";".to_string()),
            IntConst("1".to_string()),
            Symbol("+".to_string()),
            IntConst("2".to_string()),
            Symbol(";".to_string()),
            Keyword("return".to_string()),
            Symbol(";".to_string()),
            Symbol("}".to_string()),
            Symbol("}".to_string()),
        ];

        let source = r#"
            /** Prints "Hello World", File name: Main.jack */
            class Main {
                function void main() {
                    do Output.printString("Hello World !");
                    do Output.println(); // New line
                    1 + 2;
                    return; // The return statement is mandatory
                }
            }
        "#;

        assert_eq!(expected, tokenize(source));
    }
}
