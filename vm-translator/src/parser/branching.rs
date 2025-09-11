#[derive(Debug, PartialEq, Clone)]
pub struct BranchingArgs {
    pub cmd: BranchingCommand,
    pub label: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BranchingCommand {
    Label,
    Goto,
    IfGoto,
}

impl TryFrom<&str> for BranchingCommand {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "label" => Ok(BranchingCommand::Label),
            "goto" => Ok(BranchingCommand::Goto),
            "if-goto" => Ok(BranchingCommand::IfGoto),
            _ => Err("Cannot parse vm operation"),
        }
    }
}
