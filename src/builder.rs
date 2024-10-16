use crate::{BinaryOperation, UnaryOperation, CallFrame, Instruction, Register, Scope, VM};
use crate::value::Value;

#[derive(Clone, Debug)]
pub struct VMBuilder {
    pub sp: usize,
    pub scopes: Vec<Scope>,
}

macro_rules! generate_unary_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            pub fn $name(&mut self, from: Register, output: Register) -> &mut Self {
                self.add_instruction(Instruction::Unary {
                    op: UnaryOperation::$variant,
                    from,
                    output
                })
            }
        )*
    };
}

macro_rules! generate_bin_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            pub fn $name(&mut self, lhs: Register, rhs: Register, output: Register) -> &mut Self {
                self.add_instruction(Instruction::Binary {
                    op: BinaryOperation::$variant,
                    lhs,
                    rhs,
                    output
                })
            }
        )*
    };
}

impl VMBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            sp: 0,
            scopes: vec![Scope::new()],
        }
    }

    generate_bin_op_methods! {
        add_add_instruction => Add,
        add_bitand_instruction => BitAnd,
        add_bitor_instruction => BitOr,
        add_bitxor_instruction => BitXor,
        add_and_instruction => And,
        add_or_instruction => Or,
        add_xor_instruction => Xor,
        add_div_instruction => Div,
        add_mul_instruction => Mul,
        add_rem_instruction => Rem,
        add_shl_instruction => Shl,
        add_shr_instruction => Shr,
        add_sub_instruction => Sub
    }

    generate_unary_op_methods! {
        add_neg_instruction => Neg,
        add_not_instruction => Not,
        add_print_instruction => Print,
        add_eprint_instruction => EPrint
    }

    //
    pub fn enter_scope(&mut self) -> &mut Self {
        self.scopes.push(Scope::new());
        self.sp += 1;
        self
    }

    pub fn exit_scope(&mut self) -> &mut Self {
        self.sp -= 1;
        self.add_instruction(Instruction::Ret)
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.scopes[self.sp].instructions.push(instruction);
        self
    }

    pub fn add_call_instruction(&mut self, scope_id: usize) -> &mut Self {
        self.add_instruction(Instruction::Call(scope_id))
    }

    pub fn add_halt_instruction(&mut self, register: Register) -> &mut Self {
        self.add_instruction(Instruction::Halt(register))
    }

    pub fn add_copy_instruction(&mut self, from: Register, to: Register) -> &mut Self {
        self.add_instruction(Instruction::Copy(from, to))
    }

    pub fn add_load_instruction(&mut self, reg: Register, value: Value) -> &mut Self {
        self.add_instruction(Instruction::Load(reg, value))
    }

    pub fn build(&mut self) -> VM {
        VM {
            scopes: std::mem::take(&mut self.scopes),
            current: CallFrame::main(),
            frames: vec![],
            registers: Default::default(),
        }
    }

    pub fn build_multiple(&mut self) -> (VM, &mut Self) {
        let vm = VM {
            scopes: self.scopes.clone(),
            current: CallFrame::main(),
            frames: vec![],
            registers: Default::default(),
        };
        (vm, self)
    }
}