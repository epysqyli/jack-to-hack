#[derive(Debug, PartialEq)]
pub struct OperationArgs {
    pub op: Operation,
    pub segment: Option<MemorySegment>,
    pub val: Option<i16>,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
