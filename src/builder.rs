use crate::value::Value;
use crate::vm::VMOptions;
use crate::{generate_builder, generate_unary_op_methods, generate_bin_op_methods, Binary, BinaryOperation, CallFrame, Instruction, Module, Register, RigzType, Scope, Unary, UnaryOperation, VM};
use indexmap::IndexMap;
use log::Level;

#[derive(Clone, Debug)]
pub struct VMBuilder<'vm> {
    pub sp: usize,
    pub scopes: Vec<Scope<'vm>>,
    pub modules: IndexMap<&'vm str, Module<'vm>>,
    pub options: VMOptions,
}

impl<'vm> Default for VMBuilder<'vm> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'vm> VMBuilder<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self {
            sp: 0,
            scopes: vec![Scope::new()],
            modules: IndexMap::new(),
            options: VMOptions::default(),
        }
    }

    generate_builder!();

    #[inline]
    pub fn build(&mut self) -> VM<'vm> {
        VM {
            scopes: std::mem::take(&mut self.scopes),
            current: CallFrame::main(),
            frames: vec![],
            registers: Default::default(),
            lifecycles: vec![],
            modules: std::mem::take(&mut self.modules),
            sp: 0,
            options: std::mem::take(&mut self.options),
        }
    }

    #[inline]
    pub fn build_multiple(&mut self) -> (VM<'vm>, &mut Self) {
        let vm = VM {
            scopes: self.scopes.clone(),
            current: CallFrame::main(),
            frames: vec![],
            registers: Default::default(),
            lifecycles: vec![],
            modules: self.modules.clone(),
            sp: 0,
            options: self.options.clone(),
        };
        (vm, self)
    }
}
