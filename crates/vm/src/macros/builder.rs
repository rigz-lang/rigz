#[allow(unused_imports)] // for navigation & autocomplete in macro
use crate::Instruction;

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
        fn register_module(&mut self, module: impl Module<'vm> + 'static) -> &mut Self {
            self.modules.insert(module.name(), Box::new(module));
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
