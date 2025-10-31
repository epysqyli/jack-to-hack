use super::grammar::*;

pub struct CodeGenerator {
    // class level symbol table
    // method level symbol table
}

impl CodeGenerator {
    pub fn compile(class: Class) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_minimal_class() {
        let class = Class {
            name: "Main".into(),
            vars: vec![ClassVarDec {
                var_type: ClassVarType::Static,
                jack_type: JackType::Int,
                name: "a".into(),
            }],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Function,
                return_type: ReturnType::Void,
                name: "main".into(),
                parameters: vec![],
                body: SubroutineBody {
                    vars: vec![],
                    statements: vec![Statement::Return(None)],
                },
            }],
        };

        let expected: Vec<String> = vec![];
        let actual = CodeGenerator::compile(class);

        assert_eq!(expected, actual);
    }
}
