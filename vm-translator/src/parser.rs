pub mod branching;
pub mod function;
pub mod operation;
use crate::parser::{
    branching::BranchingArgs,
    operation::{MemorySegment, Operation, OperationArgs},
};

#[derive(Debug, PartialEq)]
pub enum Command {
    Branching(BranchingArgs),
    Function, // TODO: add underlying type
    Operation(OperationArgs),
}

pub fn parse(vm_command: &str) -> Command {
    let vm_tokens: Vec<&str> = vm_command.split(' ').collect();

    match vm_tokens[0] {
        "label" | "goto" | "if-goto" => Command::Branching(BranchingArgs {
            cmd: vm_tokens[0].try_into().unwrap(),
            label: vm_tokens[1].to_string(),
        }),
        "function" | "call" | "return" => Command::Function,
        "push" | "pop" | "add" | "sub" | "neg" | "gt" | "lt" | "eq" | "and" | "or" | "not" => {
            let op = Operation::try_from(vm_tokens[0]).unwrap();

            let (segment, val) = match op {
                Operation::Push | Operation::Pop => (
                    Some(MemorySegment::try_from(vm_tokens[1]).unwrap()),
                    Some(vm_tokens[2].parse::<i16>().unwrap()),
                ),
                _ => (None, None),
            };

            Command::Operation(OperationArgs { op, segment, val })
        }
        _ => panic!(""),
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_stack_push() {
        let expected = Command::Operation(OperationArgs {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: Some(1),
        });

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
            Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            Command::Operation(OperationArgs {
                op: Operation::Add,
                segment: None,
                val: None,
            }),
        ];

        assert_eq!(expected, commands);
    }

    #[test]
    fn define_label() {
        let vm_program: &str = "label SomeLabel";

        let expected_command = Command::Branching(BranchingArgs {
            cmd: branching::BranchingCommand::Label,
            label: "SomeLabel".to_string(),
        });

        assert_eq!(expected_command, parse(vm_program));
    }

    #[test]
    fn goto_label() {
        let vm_program: &str = "goto SomeLabel";

        let expected_command = Command::Branching(BranchingArgs {
            cmd: branching::BranchingCommand::Goto,
            label: "SomeLabel".to_string(),
        });

        assert_eq!(expected_command, parse(vm_program));
    }

    #[test]
    fn if_goto_label() {
        let vm_program: &str = "if-goto SomeLabel";

        let expected_command = Command::Branching(BranchingArgs {
            cmd: branching::BranchingCommand::IfGoto,
            label: "SomeLabel".to_string(),
        });

        assert_eq!(expected_command, parse(vm_program));
    }
}
