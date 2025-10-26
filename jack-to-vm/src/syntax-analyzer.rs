mod grammar;
#[path = "syntax-analyzer/parser.rs"]
mod parser;
#[path = "syntax-analyzer/parser-alt.rs"]
mod parser_alt;
#[path = "syntax-analyzer/tokenizer.rs"]
mod tokenizer;

pub fn run(jack_class: String) -> String {
    let tokens = tokenizer::tokenize(&jack_class);
    let derivation_tree = parser::Parser::parse(tokens);

    derivation_tree
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_mininal_class() {
        let input_program = r#"
            class Main {
                function void main() {
                    return;
                }
            }
        "#;

        let expected = r#"
            <class>
                <keyword>class</keyword>
                <identifier>Main</identifier>
                <symbol>{</symbol>
                <subroutineDec>
                    <keyword>function</keyword>
                    <keyword>void</keyword>
                    <identifier>main</identifier>
                    <symbol>(</symbol>
                    <symbol>)</symbol>
                    <symbol>{</symbol>
                    <subroutineBody>
                        <statements>
                            <returnStatement>
                                <keyword>return</keyword>
                                <symbol>;</symbol>
                            </returnStatement>
                        </statements>
                    </subroutineBody>
                    <symbol>}</symbol>
                </subroutineDec>
                <symbol>}</symbol>
            </class>
        "#;

        assert_eq!(
            expected.replace(" ", "").replace("\n", ""),
            super::run(input_program.into())
        );
    }
}
