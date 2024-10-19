#[macro_export]
macro_rules! generate_unary_op_methods {
    ($($name:ident => $variant:ident),*) => {
        $(
            #[inline]
            pub fn $name(&mut self, from: Register, output: Register) -> &mut Self {
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
            pub fn $name(&mut self, lhs: Register, rhs: Register, output: Register) -> &mut Self {
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
            add_lte_instruction => Lte
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
        pub fn enter_scope(&mut self) -> &mut Self {
            self.scopes.push(Scope::new());
            self.sp += 1;
            self
        }

        #[inline]
        pub fn exit_scope(&mut self, output: Register) -> &mut Self {
            let s = self.add_instruction(Instruction::Ret(output));
            s.sp -= 1;
            s
        }

        #[inline]
        pub fn add_unary_instruction(
            &mut self,
            op: UnaryOperation,
            from: Register,
            output: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::Unary(Unary { op, from, output }))
        }

        #[inline]
        pub fn add_unary_assign_instruction(
            &mut self,
            op: UnaryOperation,
            from: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::UnaryAssign(UnaryAssign { op, from }))
        }

        #[inline]
        pub fn add_unary_clear_instruction(
            &mut self,
            op: UnaryOperation,
            from: Register,
            output: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::UnaryClear(Unary { op, from, output }, Clear::One(from)))
        }

        #[inline]
        pub fn add_binary_instruction(
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
        pub fn add_binary_assign_instruction(
            &mut self,
            op: BinaryOperation,
            lhs: Register,
            rhs: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::BinaryAssign(BinaryAssign {
                op,
                lhs,
                rhs,
            }))
        }

        #[inline]
        pub fn add_binary_clear_instruction(
            &mut self,
            op: BinaryOperation,
            lhs: Register,
            rhs: Register,
            clear: Clear,
            output: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::BinaryClear(Binary {
                op,
                lhs,
                rhs,
                output,
            }, clear))
        }

        #[inline]
        pub fn register_module(&mut self, module: impl Module<'vm> + 'static) -> &mut Self {
            self.modules.insert(module.name(), Box::new(module));
            self
        }

        #[inline]
        pub fn with_options(&mut self, options: VMOptions) -> &mut Self {
            self.options = options;
            self
        }

        #[inline]
        pub fn add_instruction(&mut self, instruction: Instruction<'vm>) -> &mut Self {
            self.scopes[self.sp].instructions.push(instruction);
            self
        }

        #[inline]
        pub fn module_exists(&mut self, module: &'vm str) -> bool {
            self.modules.contains_key(module)
        }

        #[inline]
        pub fn add_call_module_instruction(
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
        pub fn add_call_extension_module_instruction(
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
        pub fn add_call_mutable_extension_module_instruction(
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
                output
            });
            self
        }

        #[inline]
        pub fn add_call_vm_extension_module_instruction(
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
        pub fn add_halt_instruction(&mut self, register: Register) -> &mut Self {
            self.add_instruction(Instruction::Halt(register))
        }

        #[inline]
        pub fn add_ret_instruction(&mut self, register: Register) -> &mut Self {
            self.add_instruction(Instruction::Ret(register))
        }

        #[inline]
        pub fn add_call_instruction(&mut self, scope_id: usize, register: Register) -> &mut Self {
            self.add_instruction(Instruction::Call(scope_id, register))
        }

        #[inline]
        pub fn add_call_eq_instruction(
            &mut self,
            lhs: Register,
            rhs: Register,
            scope_id: usize,
            register: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::CallEq(lhs, rhs, scope_id, register))
        }

        #[inline]
        pub fn add_call_neq_instruction(
            &mut self,
            lhs: Register,
            rhs: Register,
            scope_id: usize,
            register: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::CallNeq(lhs, rhs, scope_id, register))
        }

        #[inline]
        pub fn add_if_else_instruction(
            &mut self,
            truthy: Register,
            if_scope: usize,
            else_scope: usize,
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
        pub fn add_if_instruction(
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
        pub fn add_unless_instruction(
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
        pub fn add_cast_instruction(
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
        pub fn add_copy_instruction(&mut self, from: Register, to: Register) -> &mut Self {
            self.add_instruction(Instruction::Copy(from, to))
        }

        #[inline]
        pub fn add_load_instruction(&mut self, reg: Register, value: RegisterValue) -> &mut Self {
            self.add_instruction(Instruction::Load(reg, value))
        }

        #[inline]
        pub fn add_load_let_instruction(
            &mut self,
            name: &'vm str,
            register: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::LoadLetRegister(name, register))
        }

        #[inline]
        pub fn add_load_mut_instruction(
            &mut self,
            name: &'vm str,
            register: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::LoadMutRegister(name, register))
        }

        #[inline]
        pub fn add_get_variable_instruction(
            &mut self,
            name: &'vm str,
            register: Register,
        ) -> &mut Self {
            self.add_instruction(Instruction::GetVariable(name, register))
        }

        #[inline]
        pub fn add_puts_instruction(&mut self, values: Vec<Register>) -> &mut Self {
            self.add_instruction(Instruction::Puts(values))
        }

        #[inline]
        pub fn add_log_instruction(
            &mut self,
            level: Level,
            template: &'vm str,
            values: Vec<Register>,
        ) -> &mut Self {
            self.add_instruction(Instruction::Log(level, template, values))
        }
    };
}
