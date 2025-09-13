#[derive(Debug, PartialEq)]
pub enum OperationArgs {
    Push(MemorySegment, i16),
    Pop(MemorySegment, i16),
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

impl TryFrom<&str> for OperationArgs {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let vm_tokens: Vec<&str> = value.split(' ').collect();

        match vm_tokens[0] {
            "push" => Ok(OperationArgs::Push(
                vm_tokens[1].try_into().unwrap(),
                vm_tokens[2].parse::<i16>().unwrap(),
            )),
            "pop" => Ok(OperationArgs::Pop(
                vm_tokens[1].try_into().unwrap(),
                vm_tokens[2].parse::<i16>().unwrap(),
            )),
            "add" => Ok(OperationArgs::Add),
            "sub" => Ok(OperationArgs::Sub),
            "neg" => Ok(OperationArgs::Neg),
            "gt" => Ok(OperationArgs::Gt),
            "lt" => Ok(OperationArgs::Lt),
            "eq" => Ok(OperationArgs::Eq),
            "and" => Ok(OperationArgs::And),
            "or" => Ok(OperationArgs::Or),
            "not" => Ok(OperationArgs::Not),
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
