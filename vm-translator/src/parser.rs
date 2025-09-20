pub mod branching;
pub mod function;
pub mod operation;
use crate::parser::{branching::BranchingArgs, function::FunctionArgs, operation::OperationArgs};

#[derive(Debug, PartialEq)]
pub enum Command {
    Branching(BranchingArgs),
    Function(FunctionArgs),
    Operation(OperationArgs),
}

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
                let branching_args = BranchingArgs::from(vm_command, current_fn.clone()).unwrap();
                commands.push(Command::Branching(branching_args));
            }
            "function" => {
                let fn_args: FunctionArgs = vm_command.try_into().unwrap();
                current_fn = fn_args.fn_name();
                commands.push(Command::Function(fn_args));
            }
            "call" | "return" => {
                commands.push(Command::Function(vm_command.try_into().unwrap()));
            }
            "push" | "pop" | "add" | "sub" | "neg" | "gt" | "lt" | "eq" | "and" | "or" | "not" => {
                commands.push(Command::Operation(vm_command.try_into().unwrap()))
            }
            _ => panic!("Vm command cannot be parsed"),
        };
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{function::FunctionArgs, operation::MemorySegment};

    #[test]
    fn test_stack_push() {
        let expected = vec![Command::Operation(OperationArgs::Push(
            MemorySegment::Constant,
            1,
        ))];
        let vm_commands: Vec<String> = vec!["push constant 1".to_string()];
        let actual = parse(vm_commands);

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_double_stack_push_and_add() {
        let vm_commands = vec![
            "push constant 1".to_string(),
            "push constant 2".to_string(),
            "add".to_string(),
        ];

        let expected = vec![
            Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
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
            Command::Function(FunctionArgs::Function("SecondFunction".to_string(), 0)),
            Command::Branching(BranchingArgs::Label(
                "Test".to_string(),
                "SecondFunction".to_string(),
            )),
        ];

        let actual = parse(vec![
            "function FirstFunction 0".to_string(),
            "label Test".to_string(),
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
