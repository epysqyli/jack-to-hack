#[path = "syntax-analyzer/parser.rs"]
mod parser;
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
            super::run(input_program.into())
        );
    }
}
