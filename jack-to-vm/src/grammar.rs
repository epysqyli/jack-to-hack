// TODO: use enums instead of strings when reasonable

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Class {
    pub name: String,
    pub vars: Vec<ClassVarDec>,
    pub routines: Vec<SubroutineDec>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct ClassVarDec {
    pub var_type: String,  /* static|field */
    pub jack_type: String, /* int|char|boolean|class */
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct SubroutineDec {
    pub routine_type: String,       /* 'constructor'|'function'|'method' */
    pub return_type: String,        /* 'void'|type */
    pub name: String,               /* subroutineName */
    pub parameters: Vec<Parameter>, /* parameterList */
    pub body: SubroutineBody,       /* subroutineBody */
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub jack_type: String,
    pub name: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct VarDec {
    pub jack_type: String,
    pub name: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Expression {
    pub term: Term,
    pub additional: Vec<(Operation, Term)>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
