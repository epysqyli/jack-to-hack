pub mod branching;
pub mod function;
pub mod operation;

#[derive(Debug, PartialEq)]
pub enum Command {
    Branching(branching::BranchingArgs),
    Function(function::FunctionArgs),
    Operation(operation::OperationArgs),
}
