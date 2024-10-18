use crate::lifecycle::Lifecycle;
use crate::vm::RegisterValue;
use crate::vm::VMOptions;
use crate::{
    generate_bin_op_methods, generate_builder, generate_unary_op_methods, Binary, BinaryOperation,
    CallFrame, Clear, Instruction, Module, Register, RigzType, Scope, Unary, UnaryOperation, VM,
};
use indexmap::IndexMap;
use log::Level;

#[derive(Clone)]
pub struct VMBuilder<'vm> {
    pub sp: usize,
    pub scopes: Vec<Scope<'vm>>,
    pub modules: IndexMap<&'static str, Box<dyn Module<'vm>>>,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
}

impl<'vm> Default for VMBuilder<'vm> {
    #[inline]
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
            options: Default::default(),
            lifecycles: Default::default(),
        }
    }

    generate_builder!();

    #[inline]
    pub fn build(self) -> VM<'vm> {
        VM {
            scopes: self.scopes,
            current: CallFrame::main(),
            frames: vec![],
            stack: vec![],
            registers: Default::default(),
            modules: self.modules,
            sp: 0,
            options: self.options,
            lifecycles: self.lifecycles,
        }
    }
}
