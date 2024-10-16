use crate::{Register, RigzType, Value};
use log::Level;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction<'vm> {
    Halt(Register),
    Unary(Unary),
    Binary(Binary),
    Load(Register, Value<'vm>),
    Copy(Register, Register),
    Call(usize, Register),
    Log(Level, &'vm str, Vec<Register>),
    Puts(Vec<Register>),
    CallModule {
        module: &'vm str,
        function: &'vm str,
        args: Vec<Register>,
        output: Register,
    },
    CallExtensionModule {
        module: &'vm str,
        function: &'vm str,
        this: Register,
        args: Vec<Register>,
        output: Register,
    },
    CallEq(Register, Register, usize, Register),
    CallNeq(Register, Register, usize, Register),
    IfElse {
        truthy: Register,
        if_scope: usize,
        else_scope: usize,
        output: Register,
    },
    Cast {
        from: Register,
        to: Register,
        rigz_type: RigzType,
    },
    // Import(),
    // Export(),
    Ret(Register),
    GetVariable(&'vm str, Register),
    LoadLetRegister(&'vm str, Register),
    LoadMutRegister(&'vm str, Register),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unary {
    pub op: UnaryOperation,
    pub from: Register,
    pub output: Register,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
    Reverse,
    Print,
    EPrint,
    PrintLn,
    EPrintLn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOperation,
    pub lhs: Register,
    pub rhs: Register,
    pub output: Register,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shr,
    Shl,
    BitOr,
    BitAnd,
    BitXor,
    Or,
    And,
    Xor,
    Eq,
    Neq,
    Gte,
    Gt,
    Lt,
    Lte,
}
