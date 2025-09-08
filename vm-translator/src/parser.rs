#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Branching, // TODO: add underlying type
    Function,  // TODO: add underlying type
    Operation(OperationArgs),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operation {
    Push,
    Pop,
    Add,
    Sub,
    Neg,
    Gt,
    Lt,
    Eq,
    And,
    Or,
    Not,
}

impl TryFrom<&str> for Operation {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "push" => Ok(Operation::Push),
            "pop" => Ok(Operation::Pop),
            "add" => Ok(Operation::Add),
            "sub" => Ok(Operation::Sub),
            "neg" => Ok(Operation::Neg),
            "gt" => Ok(Operation::Gt),
            "lt" => Ok(Operation::Lt),
            "eq" => Ok(Operation::Eq),
            "and" => Ok(Operation::And),
            "or" => Ok(Operation::Or),
            "not" => Ok(Operation::Not),
            _ => Err("Cannot parse vm operation"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MemorySegment {
    Constant,
    Local,
    Argument,
    This,
    That,
    Pointer,
    Temp,
    Static,
}

impl TryFrom<&str> for MemorySegment {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "constant" => Ok(MemorySegment::Constant),
            "local" => Ok(MemorySegment::Local),
            "argument" => Ok(MemorySegment::Argument),
            "this" => Ok(MemorySegment::This),
            "that" => Ok(MemorySegment::That),
            "pointer" => Ok(MemorySegment::Pointer),
            "temp" => Ok(MemorySegment::Temp),
            "static" => Ok(MemorySegment::Static),
            _ => Err("Cannot parse vm memory segment"),
        }
    }
}

impl MemorySegment {
    pub fn as_asm_mnemonic(self: &Self) -> String {
        match self {
            MemorySegment::Local => "@LCL".to_string(),
            MemorySegment::Argument => "@ARG".to_string(),
            MemorySegment::This => "@THIS".to_string(),
            MemorySegment::That => "@THAT".to_string(),
            MemorySegment::Temp => "@R5".to_string(),
            _ => "".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct OperationArgs {
    pub op: Operation,
    pub segment: Option<MemorySegment>,
    pub val: Option<i16>,
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
