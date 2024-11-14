use crate::vm::RegisterValue;
use crate::vm::VMOptions;
use crate::{
    generate_bin_op_methods, generate_builder, generate_unary_op_methods, Binary, BinaryAssign,
    BinaryOperation, CallFrame, Clear, Instruction, Lifecycle, Module, Register, RigzType, Scope,
    Unary, UnaryAssign, UnaryOperation, Value, VM,
};
use indexmap::IndexMap;
use log::Level;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct VMBuilder<'vm> {
    pub sp: usize,
    pub scopes: Vec<Scope<'vm>>,
    pub modules: IndexMap<&'static str, Box<dyn Module<'vm>>>,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<Value>,
}

impl<'vm> Default for VMBuilder<'vm> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

pub trait RigzBuilder<'vm>: Debug + Default {
    fn add_constant(&mut self, value: Value) -> usize;

    fn add_instruction(&mut self, instruction: Instruction<'vm>) -> &mut Self;

    fn build(self) -> VM<'vm>;

    fn current_scope(&self) -> usize;

    fn enter_scope(&mut self, named: &'vm str) -> usize;

    fn enter_lifecycle_scope(&mut self, named: &'vm str, lifecycle: Lifecycle) -> usize;

    fn exit_scope(&mut self, current: usize, output: Register) -> &mut Self;

    fn module_exists(&mut self, module: &'vm str) -> bool;

    fn register_module(&mut self, module: impl Module<'vm> + 'static) -> &mut Self;

    fn with_options(&mut self, options: VMOptions) -> &mut Self;

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
        add_sub_instruction => Sub,
        add_gt_instruction => Gt,
        add_gte_instruction => Gte,
        add_lt_instruction => Lt,
        add_lte_instruction => Lte,
        add_elvis_instruction => Elvis
    }

    generate_unary_op_methods! {
        add_neg_instruction => Neg,
        add_not_instruction => Not,
        add_print_instruction => Print,
        add_eprint_instruction => EPrint,
        add_println_instruction => PrintLn,
        add_eprintln_instruction => EPrintLn,
        add_reverse_instruction => Reverse
    }

    #[inline]
    fn add_unary_instruction(
        &mut self,
        op: UnaryOperation,
        from: Register,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::Unary(Unary { op, from, output }))
    }

    #[inline]
    fn add_unary_assign_instruction(&mut self, op: UnaryOperation, from: Register) -> &mut Self {
        self.add_instruction(Instruction::UnaryAssign(UnaryAssign { op, from }))
    }

    #[inline]
    fn add_unary_clear_instruction(
        &mut self,
        op: UnaryOperation,
        from: Register,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::UnaryClear(
            Unary { op, from, output },
            Clear::One(from),
        ))
    }

    #[inline]
    fn add_binary_instruction(
        &mut self,
        op: BinaryOperation,
        lhs: Register,
        rhs: Register,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::Binary(Binary {
            op,
            lhs,
            rhs,
            output,
        }))
    }

    #[inline]
    fn add_binary_assign_instruction(
        &mut self,
        op: BinaryOperation,
        lhs: Register,
        rhs: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::BinaryAssign(BinaryAssign { op, lhs, rhs }))
    }

    #[inline]
    fn add_binary_clear_instruction(
        &mut self,
        op: BinaryOperation,
        lhs: Register,
        rhs: Register,
        clear: Clear,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::BinaryClear(
            Binary {
                op,
                lhs,
                rhs,
                output,
            },
            clear,
        ))
    }

    #[inline]
    fn add_call_module_instruction(
        &mut self,
        module: &'vm str,
        func: &'vm str,
        args: Vec<Register>,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallModule {
            module,
            func,
            args,
            output,
        });
        self
    }

    #[inline]
    fn add_call_extension_module_instruction(
        &mut self,
        module: &'vm str,
        func: &'vm str,
        this: Register,
        args: Vec<Register>,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallExtension {
            module,
            func,
            this,
            args,
            output,
        });
        self
    }

    #[inline]
    fn add_call_mutable_extension_module_instruction(
        &mut self,
        module: &'vm str,
        func: &'vm str,
        this: Register,
        args: Vec<Register>,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMutableExtension {
            module,
            func,
            this,
            args,
            output,
        });
        self
    }

    #[inline]
    fn add_call_vm_extension_module_instruction(
        &mut self,
        name: &'vm str,
        func: &'vm str,
        args: Vec<Register>,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallVMExtension {
            module: name,
            func,
            args,
            output,
        });
        self
    }

    #[inline]
    fn add_halt_instruction(&mut self, register: Register) -> &mut Self {
        self.add_instruction(Instruction::Halt(register))
    }

    #[inline]
    fn add_ret_instruction(&mut self, register: Register) -> &mut Self {
        self.add_instruction(Instruction::Ret(register))
    }

    #[inline]
    fn add_call_instruction(&mut self, scope_id: usize, register: Register) -> &mut Self {
        self.add_instruction(Instruction::Call(scope_id, register))
    }

    #[inline]
    fn add_call_self_instruction(
        &mut self,
        scope_id: usize,
        output: Register,
        this: Register,
        mutable: bool,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallSelf(scope_id, output, this, mutable))
    }

    #[inline]
    fn add_call_eq_instruction(
        &mut self,
        lhs: Register,
        rhs: Register,
        scope_id: usize,
        register: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallEq(lhs, rhs, scope_id, register))
    }

    #[inline]
    fn add_call_neq_instruction(
        &mut self,
        lhs: Register,
        rhs: Register,
        scope_id: usize,
        register: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallNeq(lhs, rhs, scope_id, register))
    }

    #[inline]
    fn add_if_else_instruction(
        &mut self,
        truthy: Register,
        if_scope: (usize, Register),
        else_scope: (usize, Register),
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::IfElse {
            truthy,
            if_scope,
            else_scope,
            output,
        })
    }

    #[inline]
    fn add_if_instruction(
        &mut self,
        truthy: Register,
        if_scope: usize,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::If {
            truthy,
            if_scope,
            output,
        })
    }

    #[inline]
    fn add_unless_instruction(
        &mut self,
        truthy: Register,
        unless_scope: usize,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::Unless {
            truthy,
            unless_scope,
            output,
        })
    }

    #[inline]
    fn add_cast_instruction(
        &mut self,
        from: Register,
        rigz_type: RigzType,
        to: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::Cast {
            from,
            rigz_type,
            to,
        })
    }

    #[inline]
    fn add_pop_instruction(&mut self, to: Register) -> &mut Self {
        self.add_instruction(Instruction::Pop(to))
    }

    #[inline]
    fn add_push_instruction(&mut self, from: Register) -> &mut Self {
        self.add_instruction(Instruction::Push(from))
    }

    #[inline]
    fn add_copy_instruction(&mut self, from: Register, to: Register) -> &mut Self {
        self.add_instruction(Instruction::Copy(from, to))
    }

    #[inline]
    fn add_move_instruction(&mut self, from: Register, to: Register) -> &mut Self {
        self.add_instruction(Instruction::Move(from, to))
    }

    #[inline]
    fn add_load_instruction(&mut self, reg: Register, value: RegisterValue) -> &mut Self {
        self.add_instruction(Instruction::Load(reg, value))
    }

    #[inline]
    fn add_load_let_instruction(&mut self, name: &'vm str, register: Register) -> &mut Self {
        self.add_instruction(Instruction::LoadLetRegister(name, register))
    }

    #[inline]
    fn add_set_self_instruction(&mut self, register: Register, mutable: bool) -> &mut Self {
        self.add_instruction(Instruction::SetSelf(register, mutable))
    }

    #[inline]
    fn add_get_self_instruction(&mut self, output: Register, mutable: bool) -> &mut Self {
        self.add_instruction(Instruction::GetSelf(output, mutable))
    }

    #[inline]
    fn add_load_mut_instruction(&mut self, name: &'vm str, register: Register) -> &mut Self {
        self.add_instruction(Instruction::LoadMutRegister(name, register))
    }

    #[inline]
    fn add_get_variable_instruction(&mut self, name: &'vm str, register: Register) -> &mut Self {
        self.add_instruction(Instruction::GetVariable(name, register))
    }

    #[inline]
    fn add_get_mutable_variable_instruction(
        &mut self,
        name: &'vm str,
        register: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::GetMutableVariable(name, register))
    }

    #[inline]
    fn add_puts_instruction(&mut self, values: Vec<Register>) -> &mut Self {
        self.add_instruction(Instruction::Puts(values))
    }

    #[inline]
    fn add_log_instruction(
        &mut self,
        level: Level,
        template: &'vm str,
        values: Vec<Register>,
    ) -> &mut Self {
        self.add_instruction(Instruction::Log(level, template, values))
    }

    #[inline]
    fn add_instance_get_instruction(
        &mut self,
        source: Register,
        attr: Register,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::InstanceGet(source, attr, output))
    }

    #[inline]
    fn add_instance_set_instruction(
        &mut self,
        source: Register,
        index: Register,
        value: Register,
        output: Register,
    ) -> &mut Self {
        self.add_instruction(Instruction::InstanceSet {
            source,
            index,
            value,
            output,
        })
    }
}

impl<'vm> RigzBuilder<'vm> for VMBuilder<'vm> {
    generate_builder!();

    #[inline]
    fn build(self) -> VM<'vm> {
        VM {
            scopes: self.scopes,
            current: CallFrame::main().into(),
            frames: vec![],
            modules: self.modules,
            sp: 0,
            options: self.options,
            lifecycles: self.lifecycles,
            constants: self.constants,
            stack: Default::default(),
        }
    }
}

impl<'vm> VMBuilder<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self {
            sp: 0,
            scopes: vec![Scope::default()],
            modules: IndexMap::new(),
            options: Default::default(),
            lifecycles: Default::default(),
            constants: Default::default(),
        }
    }
}
