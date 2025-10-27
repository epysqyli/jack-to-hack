#[derive(Debug, PartialEq)]
pub struct Class {
    pub name: String,
    pub vars: Vec<ClassVarDec>,
    pub routines: Vec<SubroutineDec>,
}

#[derive(Debug, PartialEq)]
pub struct ClassVarDec {
    pub var_type: ClassVarType,
    pub jack_type: JackType,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassVarType {
    Static,
    Field,
}

impl TryFrom<String> for ClassVarType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "static" => Ok(Self::Static),
            "field" => Ok(Self::Field),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum JackType {
    Int,
    Char,
    Boolean,
    Class(String),
}

impl TryFrom<String> for JackType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "int" => Ok(Self::Int),
            "char" => Ok(Self::Char),
            "boolean" => Ok(Self::Boolean),
            _ => Ok(Self::Class(value.into())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SubroutineDec {
    pub routine_type: RoutineType,
    pub return_type: ReturnType,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: SubroutineBody,
}

#[derive(Debug, PartialEq)]
pub enum ReturnType {
    Void,
    Type(JackType),
}

impl TryFrom<String> for ReturnType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "void" => Ok(Self::Void),
            _ => Ok(Self::Type(value.try_into().unwrap())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RoutineType {
    Constructor,
    Function,
    Method,
}

impl TryFrom<String> for RoutineType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "constructor" => Ok(Self::Constructor),
            "function" => Ok(Self::Function),
            "method" => Ok(Self::Method),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub jack_type: JackType,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct SubroutineBody {
    pub vars: Vec<VarDec>,
    pub statements: Vec<Statement>,
}

impl Default for SubroutineBody {
    fn default() -> Self {
        Self {
            vars: vec![],
            statements: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VarDec {
    pub jack_type: JackType,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Let {
        var_name: String,
        array_access: Option<Expression>,
        exp: Expression,
    },
    If {
        exp: Expression,
        statements: Vec<Statement>,
        else_statements: Option<Vec<Statement>>,
    },
    While {
        exp: Expression,
        statements: Vec<Statement>,
    },
    Return(Option<Expression>),
    Do(SubroutineCall),
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub term: Term,
    pub additional: Vec<(Operation, Term)>,
}

#[derive(Debug, PartialEq)]
pub enum Term {
    IntConst(usize),
    StrConst(String),
    KeywordConst(String),
    VarName(String),
    ArrayAccess {
        var_name: String,
        exp: Box<Expression>,
    },
    Expression(Box<Expression>),
    Unary {
        op: Operation,
        term: Box<Term>,
    },
    Call(SubroutineCall),
}

#[derive(Debug, PartialEq)]
pub struct SubroutineCall {
    pub callee: Option<String>, /* className or instance */
    pub routine_name: String,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Not,
    LessThan,
    GreaterThan,
    Equals,
}

impl TryFrom<String> for Operation {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "+" => Ok(Operation::Plus),
            "-" => Ok(Operation::Minus),
            "*" => Ok(Operation::Multiply),
            "/" => Ok(Operation::Divide),
            "&" => Ok(Operation::And),
            "|" => Ok(Operation::Or),
            "<" => Ok(Operation::LessThan),
            ">" => Ok(Operation::GreaterThan),
            "=" => Ok(Operation::Equals),
            "~" => Ok(Operation::Not),
            _ => Err(()),
        }
    }
}
