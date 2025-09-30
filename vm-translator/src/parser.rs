use crate::command::{
    Command, branching::BranchingArgs, function::FunctionArgs, operation::OperationArgs,
};

pub fn parse(vm_commands: Vec<String>) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut current_fn: String = "".to_string();

    for vm_command in vm_commands {
        match vm_command
            .split(' ')
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .as_ref()
        {
            "label" | "goto" | "if-goto" => {
                commands.push(Command::Branching(
					BranchingArgs::from(vm_command, current_fn.clone()).unwrap()
				));
            }
            "function" => {
                let fn_args: FunctionArgs = vm_command.try_into().unwrap();
                current_fn = fn_args.fn_name();
                commands.push(Command::Function(fn_args));
            }
            "return" | "call" => {
                commands.push(Command::Function(vm_command.try_into().unwrap()));
            }
            "push" | "pop" | "add" | "sub" | "neg" | "gt" | "lt" | "eq" | "and" | "or" | "not" => {
                commands.push(Command::Operation(
                    OperationArgs::from(vm_command, current_fn.clone()).unwrap(),
                ));
            }
            _ => panic!("Vm command {} cannot be parsed", vm_command),
        };
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::operation::MemorySegment;

    #[test]
    fn test_stack_push() {
        let expected = vec![
            Command::Function(FunctionArgs::Function(
                "TestFile.testFunction".to_string(),
                0,
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                "TestFile".to_string(),
            )),
        ];

        let vm_commands: Vec<String> = vec![
            "function TestFile.testFunction 0".to_string(),
            "push constant 1".to_string(),
        ];

        assert_eq!(expected, parse(vm_commands))
    }

    #[test]
    fn test_double_stack_push_and_add() {
        let vm_commands = vec![
            "function TestFile.testFunction 0".to_string(),
            "push constant 1".to_string(),
            "push constant 2".to_string(),
            "add".to_string(),
        ];

        let expected = vec![
            Command::Function(FunctionArgs::Function(
                "TestFile.testFunction".to_string(),
                0,
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                "TestFile".to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                "TestFile".to_string(),
            )),
            Command::Operation(OperationArgs::Add),
        ];

        assert_eq!(expected, parse(vm_commands));
    }

    #[test]
    fn define_label() {
        let expected = vec![Command::Branching(BranchingArgs::Label(
            "SomeLabel".to_string(),
            "".to_string(),
        ))];

        let actual = parse(vec!["label SomeLabel".to_string()]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn goto_label() {
        let expected = vec![Command::Branching(BranchingArgs::Goto(
            "SomeLabel".to_string(),
            "".to_string(),
        ))];

        let actual = parse(vec!["goto SomeLabel".to_string()]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn if_goto_label() {
        let expected = vec![Command::Branching(BranchingArgs::IfGoto(
            "SomeLabel".to_string(),
            "".to_string(),
        ))];

        let actual = parse(vec!["if-goto SomeLabel".to_string()]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn identical_labels_are_different_within_different_functions() {
        let expected = vec![
            Command::Function(FunctionArgs::Function("FirstFunction".to_string(), 0)),
            Command::Branching(BranchingArgs::Label(
                "Test".to_string(),
                "FirstFunction".to_string(),
            )),
            Command::Function(FunctionArgs::Return),
            Command::Function(FunctionArgs::Function("SecondFunction".to_string(), 0)),
            Command::Branching(BranchingArgs::Label(
                "Test".to_string(),
                "SecondFunction".to_string(),
            )),
        ];

        let actual = parse(vec![
            "function FirstFunction 0".to_string(),
            "label Test".to_string(),
            "return".to_string(),
            "function SecondFunction 0".to_string(),
            "label Test".to_string(),
        ]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn define_function() {
        let expected = vec![Command::Function(FunctionArgs::Function(
            "TestFunc".to_string(),
            2,
        ))];

        let actual = parse(vec!["function TestFunc 2".to_string()]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn call_function() {
        let expected = vec![Command::Function(FunctionArgs::Call(
            "TestFunc".to_string(),
            2,
        ))];

        let actual = parse(vec!["call TestFunc 2".to_string()]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn return_from_function() {
        let expected = vec![Command::Function(FunctionArgs::Return)];

        let actual = parse(vec!["return".to_string()]);

        assert_eq!(expected, actual);
    }
}
