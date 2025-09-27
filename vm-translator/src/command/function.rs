#[derive(Debug, PartialEq, Clone)]
pub enum FunctionArgs {
    Function(String, u8),
    Call(String, u8),
    Return,
}

impl TryFrom<String> for FunctionArgs {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let tokens: Vec<&str> = value.split(' ').collect();

        match tokens[0] {
            "function" => Ok(FunctionArgs::Function(
                tokens[1].to_string(),
                tokens[2].parse::<u8>().expect("Missing function nVars"),
            )),
            "call" => Ok(FunctionArgs::Call(
                tokens[1].to_string(),
                tokens[2].parse::<u8>().expect("Missing call nArgs"),
            )),
            "return" => Ok(FunctionArgs::Return),
            _ => Err("Cannot parse vm operation"),
        }
    }
}

impl FunctionArgs {
    pub fn fn_name(self: &Self) -> String {
        match self {
            FunctionArgs::Function(name, _) => name.to_string(),
            _ => panic!("Trying to extract current function name from non 'function' command"),
        }
    }
}
