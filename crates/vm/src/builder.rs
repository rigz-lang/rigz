use crate::vm::VMOptions;
use crate::{
    BinaryOperation, BroadcastArgs, Instruction, Lifecycle, LoadValue, Module, RigzType, Scope,
    UnaryOperation, Value, VM,
};
use dashmap::DashMap;
use log::Level;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct VMBuilder<'vm> {
    pub sp: usize,
    pub scopes: Vec<Scope<'vm>>,
    pub modules: HashMap<&'static str, Arc<dyn Module<'vm> + Send + Sync>>,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<Value>,
}

impl Default for VMBuilder<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            sp: 0,
            scopes: vec![Default::default()],
            modules: Default::default(),
            options: Default::default(),
            lifecycles: vec![],
            constants: vec![],
        }
    }
}

#[macro_export]
macro_rules! generate_unary_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            fn $name(&mut self) -> &mut Self {
                self.add_instruction(Instruction::Unary(UnaryOperation::$variant))
            }
        )*
    };
}

#[macro_export]
macro_rules! generate_bin_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            fn $name(&mut self) -> &mut Self {
                self.add_instruction(Instruction::Binary(BinaryOperation::$variant))
            }
        )*
    };
}

pub trait RigzBuilder<'vm>: Debug + Default {
    fn add_constant(&mut self, value: Value) -> usize;

    fn add_instruction(&mut self, instruction: Instruction<'vm>) -> &mut Self;

    fn build(self) -> VM<'vm>;

    fn current_scope(&self) -> usize;

    fn enter_scope(
        &mut self,
        named: &'vm str,
        args: Vec<(&'vm str, bool)>,
        set_self: Option<bool>,
    ) -> usize;

    fn enter_lifecycle_scope(
        &mut self,
        named: &'vm str,
        lifecycle: Lifecycle,
        args: Vec<(&'vm str, bool)>,
        set_self: Option<bool>,
    ) -> usize;

    fn exit_scope(&mut self, current: usize) -> &mut Self;

    fn convert_to_lazy_scope(&mut self, scope_id: usize, var: &'vm str) -> &mut Self;

    fn module_exists(&mut self, module: &'vm str) -> bool;

    fn register_module(&mut self, module: impl Module<'vm> + 'static + Send + Sync) -> &mut Self;

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

    fn add_for_list_instruction(&mut self, scope: usize) -> &mut Self {
        self.add_instruction(Instruction::ForList { scope })
    }

    fn add_for_map_instruction(&mut self, scope: usize) -> &mut Self {
        self.add_instruction(Instruction::ForMap { scope })
    }

    #[inline]
    fn add_unary_instruction(&mut self, op: UnaryOperation) -> &mut Self {
        self.add_instruction(Instruction::Unary(op))
    }

    #[inline]
    fn add_binary_instruction(&mut self, op: BinaryOperation) -> &mut Self {
        self.add_instruction(Instruction::Binary(op))
    }

    #[inline]
    fn add_binary_assign_instruction(&mut self, op: BinaryOperation) -> &mut Self {
        self.add_instruction(Instruction::BinaryAssign(op))
    }

    #[inline]
    fn add_call_module_instruction(
        &mut self,
        module: &'vm str,
        func: &'vm str,
        args: usize,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallModule { module, func, args });
        self
    }

    #[inline]
    fn add_call_extension_module_instruction(
        &mut self,
        module: &'vm str,
        func: &'vm str,
        args: usize,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallExtension { module, func, args });
        self
    }

    #[inline]
    fn add_call_mutable_extension_module_instruction(
        &mut self,
        module: &'vm str,
        func: &'vm str,
        args: usize,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMutableExtension { module, func, args });
        self
    }

    #[inline]
    fn add_call_vm_extension_module_instruction(
        &mut self,
        name: &'vm str,
        func: &'vm str,
        args: usize,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallVMExtension {
            module: name,
            func,
            args,
        });
        self
    }

    #[inline]
    fn add_halt_instruction(&mut self) -> &mut Self {
        self.add_instruction(Instruction::Halt)
    }

    #[inline]
    fn add_ret_instruction(&mut self) -> &mut Self {
        self.add_instruction(Instruction::Ret)
    }

    #[inline]
    fn add_call_instruction(&mut self, scope: usize) -> &mut Self {
        self.add_instruction(Instruction::Call(scope))
    }

    #[inline]
    fn add_call_memo_instruction(&mut self, scope: usize) -> &mut Self {
        self.add_instruction(Instruction::CallMemo(scope))
    }

    #[inline]
    fn add_call_eq_instruction(&mut self, scope_id: usize) -> &mut Self {
        self.add_instruction(Instruction::CallEq(scope_id))
    }

    #[inline]
    fn add_call_neq_instruction(&mut self, scope_id: usize) -> &mut Self {
        self.add_instruction(Instruction::CallNeq(scope_id))
    }

    #[inline]
    fn add_if_else_instruction(&mut self, if_scope: usize, else_scope: usize) -> &mut Self {
        self.add_instruction(Instruction::IfElse {
            if_scope,
            else_scope,
        })
    }

    #[inline]
    fn add_if_instruction(&mut self, if_scope: usize) -> &mut Self {
        self.add_instruction(Instruction::If(if_scope))
    }

    #[inline]
    fn add_unless_instruction(&mut self, unless_scope: usize) -> &mut Self {
        self.add_instruction(Instruction::Unless(unless_scope))
    }

    #[inline]
    fn add_cast_instruction(&mut self, rigz_type: RigzType) -> &mut Self {
        self.add_instruction(Instruction::Cast { rigz_type })
    }

    #[inline]
    fn add_pop_instruction(&mut self, amount: usize) -> &mut Self {
        self.add_instruction(Instruction::Pop(amount))
    }

    #[inline]
    fn add_load_instruction(&mut self, value: LoadValue) -> &mut Self {
        self.add_instruction(Instruction::Load(value))
    }

    #[inline]
    fn add_get_variable_reference_instruction(&mut self, name: &'vm str) -> &mut Self {
        self.add_instruction(Instruction::GetVariableReference(name))
    }

    #[inline]
    fn add_get_variable_instruction(&mut self, name: &'vm str) -> &mut Self {
        self.add_instruction(Instruction::GetVariable(name))
    }

    #[inline]
    fn add_get_mutable_variable_instruction(&mut self, name: &'vm str) -> &mut Self {
        self.add_instruction(Instruction::GetMutableVariable(name))
    }

    #[inline]
    fn add_get_self_instruction(&mut self) -> &mut Self {
        self.add_instruction(Instruction::GetVariable("self"))
    }

    #[inline]
    fn add_get_self_mut_instruction(&mut self) -> &mut Self {
        self.add_instruction(Instruction::GetMutableVariable("self"))
    }

    #[inline]
    fn add_load_let_instruction(&mut self, name: &'vm str) -> &mut Self {
        self.add_instruction(Instruction::LoadLet(name))
    }

    #[inline]
    fn add_load_mut_instruction(&mut self, name: &'vm str) -> &mut Self {
        self.add_instruction(Instruction::LoadMut(name))
    }

    #[inline]
    fn add_puts_instruction(&mut self, values: usize) -> &mut Self {
        self.add_instruction(Instruction::Puts(values))
    }

    #[inline]
    fn add_log_instruction(
        &mut self,
        level: Level,
        template: &'vm str,
        values: usize,
    ) -> &mut Self {
        self.add_instruction(Instruction::Log(level, template, values))
    }

    #[inline]
    fn add_instance_get_instruction(&mut self, multiple: bool) -> &mut Self {
        self.add_instruction(Instruction::InstanceGet(multiple))
    }

    #[inline]
    fn add_instance_set_instruction(&mut self) -> &mut Self {
        self.add_instruction(Instruction::InstanceSet)
    }

    #[inline]
    fn add_send_instruction(&mut self, args: usize) -> &mut Self {
        self.add_instruction(Instruction::Send(args))
    }

    #[inline]
    fn add_receive_instruction(&mut self, args: usize) -> &mut Self {
        self.add_instruction(Instruction::Receive(args))
    }

    #[inline]
    fn add_spawn_instruction(&mut self, scope_id: usize, timeout: bool) -> &mut Self {
        self.add_instruction(Instruction::Spawn(scope_id, timeout))
    }

    #[inline]
    fn add_broadcast_instruction(&mut self, args: BroadcastArgs) -> &mut Self {
        self.add_instruction(Instruction::Broadcast(args))
    }

    #[inline]
    fn add_sleep_instruction(&mut self) -> &mut Self {
        self.add_instruction(Instruction::Sleep)
    }
}

#[macro_export]
macro_rules! generate_builder {
    () => {
        fn current_scope(&self) -> usize {
            self.sp
        }

        #[inline]
        fn enter_scope(
            &mut self,
            named: &'vm str,
            args: Vec<(&'vm str, bool)>,
            set_self: Option<bool>,
        ) -> usize {
            let next = self.scopes.len();
            self.scopes.push(Scope::new(named, args, set_self));
            self.sp = self.scopes.len() - 1;
            next
        }

        #[inline]
        fn convert_to_lazy_scope(&mut self, scope_id: usize, variable: &'vm str) -> &mut Self {
            let scope = &mut self.scopes[scope_id];
            let last = scope.instructions.len() - 1;
            scope
                .instructions
                .insert(last, Instruction::PersistScope(variable));
            self
        }

        #[inline]
        fn enter_lifecycle_scope(
            &mut self,
            named: &'vm str,
            lifecycle: Lifecycle,
            args: Vec<(&'vm str, bool)>,
            set_self: Option<bool>,
        ) -> usize {
            let next = self.scopes.len();
            self.scopes
                .push(Scope::lifecycle(named, args, lifecycle, set_self));
            self.sp = next;
            next
        }

        #[inline]
        fn exit_scope(&mut self, current: usize) -> &mut Self {
            let s = self.add_instruction(Instruction::Ret);
            s.sp = current;
            s
        }

        #[inline]
        fn register_module(
            &mut self,
            module: impl Module<'vm> + 'static + Send + Sync,
        ) -> &mut Self {
            self.modules.insert(module.name(), Arc::new(module));
            self
        }

        #[inline]
        fn with_options(&mut self, options: VMOptions) -> &mut Self {
            self.options = options;
            self
        }

        #[inline]
        fn add_instruction(&mut self, instruction: Instruction<'vm>) -> &mut Self {
            self.scopes[self.sp].instructions.push(instruction);
            self
        }

        #[inline]
        fn module_exists(&mut self, module: &'vm str) -> bool {
            self.modules.contains_key(module)
        }

        #[inline]
        fn add_constant(&mut self, value: Value) -> usize {
            let index = self.constants.len();
            self.constants.push(value);
            index
        }
    };
}

impl<'vm> RigzBuilder<'vm> for VMBuilder<'vm> {
    generate_builder!();

    #[inline]
    fn build(self) -> VM<'vm> {
        VM {
            scopes: self.scopes,
            modules: Arc::new(DashMap::from_iter(self.modules)),
            options: self.options,
            lifecycles: self.lifecycles,
            constants: self.constants,
            ..Default::default()
        }
    }
}

impl VMBuilder<'_> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}
