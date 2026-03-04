#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ClassDecl {
        name: String,
        members: Vec<ClassMember>,
    },
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: FunctionBody,
    },
    ForLoop {
        item: String,
        index: Option<String>,
        collection: Expression,
        body: Vec<Statement>,
    },
    ForClassic {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Vec<Statement>,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Return(Option<Expression>),
    Throw(Option<Expression>),
    TryCatch {
        try_branch: Vec<Statement>,
        catches: Vec<CatchBlock>,
        finally_branch: Option<Vec<Statement>>,
    },
    VariableDecl {
        name: String,
        value: Expression,
    },
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchBlock {
    pub exception_var: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Property(String),
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    New {
        class_name: String,
        args: Vec<Expression>,
    },
    Assignment {
        target: AssignmentTarget,
        value: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    FunctionCall {
        base: Box<Expression>,
        args: Vec<Expression>,
    },
    ArrayAccess {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    MemberAccess {
        base: Box<Expression>,
        member: String,
    },
    Prefix {
        operator: String,
        target: AssignmentTarget,
    },
    Postfix {
        base: Box<Expression>,
        operator: String,
    },
    Identifier(String),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Identifier(String),
    Member {
        base: Box<Expression>,
        member: String,
    },
    Index {
        base: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(Vec<StringPart>),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<Expression>),
    Struct(Vec<(Expression, Expression)>),
    Function {
        params: Vec<String>,
        body: FunctionBody,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Block(Vec<Statement>),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expression(Expression),
}
