use crate::{Register, RigzType, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction<'vm> {
    Halt(Register),
    Unary {
        op: UnaryOperation,
        from: Register,
        output: Register,
    },
    Binary {
        op: BinaryOperation,
        lhs: Register,
        rhs: Register,
        output: Register,
    },
    Load(Register, Value<'vm>),
    Copy(Register, Register),
    Call(usize, Register),
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
        output: Register
    },
    Cast {
        from: Register,
        to: Register,
        rigz_type: RigzType,
    },
    // Import(),
    // Export(),
    Ret(Register),
    GetVariable(String, Register),
    LoadLetRegister(String, Register),
    LoadMutRegister(String, Register),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
    Rev,
    Print,
    EPrint,
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