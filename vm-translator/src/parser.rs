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

pub fn parse(vm_command: &str) -> Command {
    match vm_command
        .split(' ')
        .collect::<Vec<&str>>()
        .first()
        .unwrap()
        .as_ref()
    {
        "label" | "goto" | "if-goto" => Command::Branching(vm_command.try_into().unwrap()),
        "function" | "call" | "return" => Command::Function(vm_command.try_into().unwrap()),
        "push" | "pop" | "add" | "sub" | "neg" | "gt" | "lt" | "eq" | "and" | "or" | "not" => {
            Command::Operation(vm_command.try_into().unwrap())
        }
        _ => panic!("Vm command cannot be parsed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{function::FunctionArgs, operation::MemorySegment};

    #[test]
    fn test_stack_push() {
        let expected = Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1));
        assert_eq!(expected, parse("push constant 1"))
    }

    #[test]
    fn test_double_stack_push_and_add() {
        let vm_program: Vec<&str> = vec!["push constant 1", "push constant 2", "add"];

        let commands: Vec<Command> = vm_program
            .iter()
            .map(|vm_command| parse(vm_command))
            .collect();

        let expected = vec![
            Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            Command::Operation(OperationArgs::Add),
        ];

        assert_eq!(expected, commands);
    }

    #[test]
    fn define_label() {
        let expected_command = Command::Branching(BranchingArgs::Label("SomeLabel".to_string()));
        assert_eq!(expected_command, parse("label SomeLabel"));
    }

    #[test]
    fn goto_label() {
        let expected_command = Command::Branching(BranchingArgs::Goto("SomeLabel".to_string()));
        assert_eq!(expected_command, parse("goto SomeLabel"));
    }

    #[test]
    fn if_goto_label() {
        let expected_command = Command::Branching(BranchingArgs::IfGoto("SomeLabel".to_string()));
        assert_eq!(expected_command, parse("if-goto SomeLabel"));
    }

    #[test]
    fn define_function() {
        let expected_command = Command::Function(FunctionArgs::Function("TestFunc".to_string(), 2));
        assert_eq!(expected_command, parse("function TestFunc 2"));
    }

    #[test]
    fn call_function() {
        let expected_command = Command::Function(FunctionArgs::Call("TestFunc".to_string(), 2));
        assert_eq!(expected_command, parse("call TestFunc 2"));
    }

    #[test]
    fn return_from_function() {
        let expected_command = Command::Function(FunctionArgs::Return);
        assert_eq!(expected_command, parse("return"));
    }
}
