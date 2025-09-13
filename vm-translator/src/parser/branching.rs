#[derive(Debug, PartialEq, Clone)]
pub enum BranchingArgs {
    Label(String),
    Goto(String),
    IfGoto(String),
}

impl TryFrom<&str> for BranchingArgs {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let cmd_and_label: Vec<&str> = value.split(' ').collect();
        let cmd = cmd_and_label[0];
        let label = cmd_and_label[1];

        match cmd {
            "label" => Ok(BranchingArgs::Label(label.to_string())),
            "goto" => Ok(BranchingArgs::Goto(label.to_string())),
            "if-goto" => Ok(BranchingArgs::IfGoto(label.to_string())),
            _ => Err("Cannot parse vm operation"),
        }
    }
}
