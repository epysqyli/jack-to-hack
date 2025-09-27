#[derive(Debug, PartialEq, Clone)]
pub enum BranchingArgs {
    Label(String, String),
    Goto(String, String),
    IfGoto(String, String),
}

impl BranchingArgs {
    pub fn from(vm_command: String, fn_name: String) -> Result<Self, &'static str> {
        let cmd_and_label: Vec<&str> = vm_command.split(' ').collect();
        let cmd = cmd_and_label[0];
        let label = cmd_and_label[1];

        match cmd {
            "label" => Ok(BranchingArgs::Label(label.to_string(), fn_name)),
            "goto" => Ok(BranchingArgs::Goto(label.to_string(), fn_name)),
            "if-goto" => Ok(BranchingArgs::IfGoto(label.to_string(), fn_name)),
            _ => Err("Cannot parse vm operation"),
        }
    }
}
