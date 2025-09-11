pub mod operation;
use crate::parser::operation::{MemorySegment, Operation, OperationArgs};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Branching, // TODO: add underlying type
    Function,  // TODO: add underlying type
    Operation(OperationArgs),
}

pub fn parse(vm_command: &str) -> Command {
    let vm_tokens: Vec<&str> = vm_command.split(' ').collect();

    match vm_tokens[0] {
        "label" | "goto" | "if-goto" => Command::Branching,
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
}
