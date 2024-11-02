#[allow(unused_imports)] // for navigation & autocomplete in macro
use crate::Instruction;

#[macro_export]
macro_rules! generate_unary_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            fn $name(&mut self, from: Register, output: Register) -> &mut Self {
                self.add_instruction(Instruction::Unary(Unary {
                    op: UnaryOperation::$variant,
                    from,
                    output
                }))
            }
        )*
    };
}

#[macro_export]
macro_rules! generate_bin_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            fn $name(&mut self, lhs: Register, rhs: Register, output: Register) -> &mut Self {
                self.add_instruction(Instruction::Binary(Binary {
                    op: BinaryOperation::$variant,
                    lhs,
                    rhs,
                    output
                }))
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
        fn enter_scope(&mut self, named: &'vm str) -> usize {
            let next = self.scopes.len();
            self.scopes.push(Scope::new(named));
            self.sp = self.scopes.len() - 1;
            next
        }

        #[inline]
        fn enter_lifecycle_scope(&mut self, named: &'vm str, lifecycle: Lifecycle) -> usize {
            let next = self.scopes.len();
            self.scopes.push(Scope::lifecycle(named, lifecycle));
            self.sp = next;
            next
        }

        #[inline]
        fn exit_scope(&mut self, current: usize, output: Register) -> &mut Self {
            let s = self.add_instruction(Instruction::Ret(output));
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
