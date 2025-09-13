#[derive(Debug, PartialEq, Clone)]
pub enum FunctionArgs {
    Function(String, u8),
    Call(String, u8),
    Return,
}

impl TryFrom<&str> for FunctionArgs {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens: Vec<&str> = value.split(' ').collect();

        match tokens[0] {
            "function" => Ok(FunctionArgs::Function(
                tokens[1].to_string(),
                tokens[2].parse::<u8>().unwrap(),
            )),
            "call" => Ok(FunctionArgs::Call(
                tokens[1].to_string(),
                tokens[2].parse::<u8>().unwrap(),
            )),
            "return" => Ok(FunctionArgs::Return),
            _ => Err("Cannot parse vm operation"),
        }
    }
}
