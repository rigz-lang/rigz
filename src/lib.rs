extern crate core;

mod builder;
mod instructions;
mod macros;
mod number;
mod value;

use indexmap::map::Entry;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::string::ToString;

pub use builder::VMBuilder;
pub use instructions::{Instruction, BinaryOperation, UnaryOperation};
pub use number::Number;
pub use value::Value;

pub trait Rev {
    type Output;

    fn rev(self) -> Self::Output;
}

pub trait Logical<Rhs> {
    type Output;

    fn and(self, rhs: Rhs) -> Self::Output;
    fn or(self, rhs: Rhs) -> Self::Output;
    fn xor(self, rhs: Rhs) -> Self::Output;
}

pub type Register = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VMError {
    RuntimeError(String),
    FrameError(String),
    ScopeError(String),
    InvalidPC(String),
    EmptyRegister(String),
    ConversionError(String),
    ScopeDoesNotExist(String),
    UnsupportedOperation(String),
    ParseError(String, usize, usize),
    VariableDoesNotExist(String),
    InvalidModule(String),
    InvalidModuleFunction(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RigzType {
    None,
    Any,
    Bool,
    Int,
    Float,
    Number,
    UInt,
    String,
    List,
    Map,
    Error,
    Function(Vec<RigzType>, Box<RigzType>),
    Object(RigzObjectDefinition),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigzObjectDefinition {
    pub name: String,
    pub fields: IndexMap<String, RigzType>,
}

impl Hash for RigzObjectDefinition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (k, v) in &self.fields {
            k.hash(state);
            v.hash(state);
        }
    }
}

static NONE: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "None".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::None)]),
});

static BOOL: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Bool".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Bool)]),
});

static NUMBER: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Number".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Number)]),
});

static STRING: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "String".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::String)]),
});

static ERROR: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Error".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Error)]),
});

static LIST: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "List".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::List)]),
});

static MAP: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Map".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Map)]),
});

#[derive(Clone, Debug, PartialEq)]
pub struct RigzObject<'vm> {
    pub fields: IndexMap<String, Value<'vm>>,
    pub definition_index: &'vm RigzObjectDefinition,
}

impl<'vm> RigzObject<'vm> {
    pub fn cast(&self, def: RigzObjectDefinition) -> Result<RigzObject<'vm>, VMError> {
        if self.definition_index == &def {
            return Ok(self.clone());
        }

        if self.definition_index.fields == def.fields {
            return Ok(self.clone());
        }

        Err(VMError::ConversionError(format!(
            "Cannot convert {} to {}",
            self.definition_index.name, def.name
        )))
    }
}

impl<'vm> RigzObject<'vm> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn equivalent(&self, other: &IndexMap<Value<'vm>, Value<'vm>>) -> bool {
        for (k, v) in other {
            let key = k.to_string();
            if !self.fields.contains_key(&key) {
                return false;
            }
            match self.fields.get(&key) {
                None => return false,
                Some(o) => {
                    if !o.eq(v) {
                        return false;
                    }
                }
            };
        }
        true
    }
}

impl<'vm> Hash for RigzObject<'vm> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.definition_index.name.hash(state);
        for (k, v) in &self.fields {
            k.hash(state);
            v.hash(state);
        }
    }
}

#[derive(Clone, Debug)]
pub enum Variable {
    Let(Register),
    Mut(Register),
}

#[derive(Clone, Debug)]
pub struct CallFrame {
    pub scope_id: usize,
    pub pc: usize,
    pub variables: IndexMap<String, Variable>, // TODO switch to intern strings
    pub parent: Option<usize>,
    pub output: Register,
}

impl<'vm> CallFrame {
    pub(crate) fn get_variable(&self, name: &String, vm: &VM<'vm>) -> Option<Register> {
        match self.variables.get(name) {
            None => match self.parent {
                None => None,
                Some(parent) => vm.frames[parent].get_variable(name, vm),
            },
            Some(v) => match v {
                Variable::Let(v) => Some(*v),
                Variable::Mut(v) => Some(*v),
            },
        }
    }
}

impl Default for CallFrame {
    fn default() -> Self {
        Self::main()
    }
}

impl<'vm> CallFrame {
    fn next_instruction(&mut self, scope: &Scope<'vm>) -> Instruction<'vm> {
        let instruction = scope.instructions[self.pc].clone();
        self.pc += 1;
        instruction
    }

    pub fn main() -> Self {
        Self {
            output: 0,
            scope_id: 0,
            pc: 0,
            variables: Default::default(),
            parent: None,
        }
    }

    pub fn child(scope_id: usize, parent: usize, output: Register) -> Self {
        Self {
            scope_id,
            output,
            pc: 0,
            variables: Default::default(),
            parent: Some(parent),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope<'vm> {
    pub instructions: Vec<Instruction<'vm>>,
    pub type_definitions: IndexMap<String, RigzObjectDefinition>,
}

impl<'vm> Scope<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub struct Lifecycle<'vm> {
    pub name: String,
    pub parent: Option<&'vm Lifecycle<'vm>>,
}

#[derive(Clone, Debug)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub current: CallFrame,
    pub frames: Vec<CallFrame>,
    pub registers: IndexMap<usize, Value<'vm>>,
    pub lifecycles: Vec<Lifecycle<'vm>>,
    pub modules: IndexMap<&'vm str, Module<'vm>>,
    pub sp: usize,
}

#[derive(Clone, Debug)]
pub struct Module<'vm> {
    pub name: &'vm str,
    pub functions: IndexMap<&'vm str, fn(Vec<Value<'vm>>) -> Value<'vm>>,
    pub extension_functions:
        IndexMap<RigzType, IndexMap<&'vm str, fn(Value<'vm>, Vec<Value<'vm>>) -> Value<'vm>>>,
}

impl<'vm> Default for VM<'vm> {
    fn default() -> Self {
        VM::new()
    }
}

pub enum VMState<'vm> {
    Running,
    Done(Value<'vm>)
}

impl<'vm> VM<'vm> {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            current: Default::default(),
            frames: vec![],
            registers: Default::default(),
            lifecycles: vec![],
            modules: Default::default(),
            sp: 0,
        }
    }

    generate_builder!();

    pub fn insert_register(&mut self, register: Register, value: Value<'vm>) {
        if register <= 1 {
            return;
        }

        self.registers.insert(register, value);
    }

    pub fn get_register(&mut self, register: Register) -> Result<Value<'vm>, VMError> {
        if register == 0 {
            return Ok(Value::None);
        }

        if register == 1 {
            return Ok(Value::Number(Number::Int(1)));
        }

        /**
        a = do
          1 + 2
        end
        */
        let v = match self.registers.get(&register) {
            None => return Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => {
                v.clone()
            },
        };

        if let Value::ScopeId(scope, output) = v {
            self.call_frame(scope, output)?;
            self.run()
        } else {
            Ok(v)
        }
    }

    pub fn process_instruction(&mut self, instruction: Instruction<'vm>) -> Result<VMState<'vm>, VMError> {
        match instruction {
            Instruction::Halt(r) => return Ok(VMState::Done(self.get_register(r)?)),
            Instruction::Unary { op, from, output } => {
                let val = match self.registers.shift_remove(&from) {
                    None => return Err(VMError::EmptyRegister(format!("R{} is empty", from))),
                    Some(v) => v,
                };
                match op {
                    UnaryOperation::Neg => {
                        self.insert_register(output, val);
                    }
                    UnaryOperation::Not => {
                        self.insert_register(output, val);
                    }
                    UnaryOperation::Print => {
                        println!("{}", val);
                        self.insert_register(output, val);
                    }
                    UnaryOperation::EPrint => {
                        eprintln!("{}", val);
                        self.insert_register(output, val);
                    }
                    UnaryOperation::Rev => {
                        self.insert_register(output, val.rev());
                    }
                }
            }
            Instruction::Binary {
                op,
                lhs,
                rhs,
                output,
            } => {
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                let v = match op {
                    BinaryOperation::Add => lhs + rhs,
                    BinaryOperation::Sub => lhs - rhs,
                    BinaryOperation::Shr => lhs >> rhs,
                    BinaryOperation::Shl => lhs << rhs,
                    BinaryOperation::Eq => Value::Bool(lhs == rhs),
                    BinaryOperation::Neq => Value::Bool(lhs != rhs),
                    BinaryOperation::Mul => lhs * rhs,
                    BinaryOperation::Div => lhs / rhs,
                    BinaryOperation::Rem => lhs % rhs,
                    BinaryOperation::BitOr => lhs | rhs,
                    BinaryOperation::BitAnd => lhs & rhs,
                    BinaryOperation::BitXor => lhs ^ rhs,
                    BinaryOperation::And => lhs.and(rhs),
                    BinaryOperation::Or => lhs.or(rhs),
                    BinaryOperation::Xor => lhs.xor(rhs),
                    BinaryOperation::Gt => Value::Bool(lhs > rhs),
                    BinaryOperation::Gte => Value::Bool(lhs >= rhs),
                    BinaryOperation::Lt => Value::Bool(lhs < rhs),
                    BinaryOperation::Lte => Value::Bool(lhs <= rhs),
                };

                self.insert_register(output, v);
            }
            Instruction::Load(r, v) => {
                self.insert_register(r, v);
            }
            Instruction::LoadLetRegister(name, register) => {
                self.load_let(name, register)?;
            }
            Instruction::LoadMutRegister(name, register) => {
                self.load_mut(name, register)?;
            }
            Instruction::Copy(from, to) => {
                let copy = match self.registers.get(&from) {
                    None => return Err(VMError::EmptyRegister(format!("R{} is empty", from))),
                    Some(s) => s.clone(),
                };
                self.insert_register(to, copy);
            }
            Instruction::Call(scope_index, register) => {
                self.call_frame(scope_index, register)?;
            }
            Instruction::Ret(output) => {
                let current = self.current.output;
                let source = self.get_register(current)?;
                match self.frames.pop() {
                    None => return Ok(VMState::Done(source)),
                    Some(c) => {
                        self.insert_register(output, source);
                        let variables = std::mem::take(&mut self.current.variables);
                        for reg in variables.values() {
                            let _ = match reg {
                                Variable::Let(r) => self.get_register(*r)?,
                                Variable::Mut(r) => self.get_register(*r)?,
                            };
                        }
                        self.current = c;
                    }
                }
            },
            Instruction::Cast {
                from,
                rigz_type,
                to,
            } => {
                let value = self.get_register(from)?;
                self.insert_register(to, value.cast(rigz_type)?);
            }
            Instruction::CallEq(a, b, scope_index, output) => {
                let a = self.get_register(a)?;
                let b = self.get_register(b)?;
                if a == b {
                    self.call_frame(scope_index, output)?;
                }
            }
            Instruction::CallNeq(a, b, scope_index, output) => {
                let a = self.get_register(a)?;
                let b = self.get_register(b)?;
                if a != b {
                    self.call_frame(scope_index, output)?;
                }
            }
            Instruction::IfElse {
                truthy,
                if_scope,
                else_scope,
                output
            } => {
                if self.get_register(truthy)?.to_bool() {
                    self.call_frame(if_scope, output)?;
                } else {
                    self.call_frame(else_scope, output)?;
                }
            }
            Instruction::GetVariable(name, reg) => {
                match self.current.get_variable(&name, self) {
                    None => {
                        return Err(VMError::VariableDoesNotExist(format!(
                            "Variable {} does not exist",
                            name
                        )))
                    }
                    Some(s) => match self.registers.get(&s) {
                        None => {
                            return Err(VMError::EmptyRegister(format!(
                                "Register {} does not exist",
                                s
                            )))
                        }
                        Some(v) => self.insert_register(reg, v.clone()),
                    },
                }
            }
            Instruction::CallModule {
                module,
                function,
                args,
                output,
            } => {
                let f = match self.modules.get(module) {
                    None => {
                        return Err(VMError::InvalidModule(format!(
                            "Module {} does not exist",
                            module
                        )))
                    }
                    Some(m) => match m.functions.get(function) {
                        None => {
                            return Err(VMError::InvalidModuleFunction(format!(
                                "Module {}.{} does not exist",
                                module, function
                            )))
                        }
                        Some(f) => {
                            f.clone()
                        }
                    },
                };
                let mut inner_args = Vec::with_capacity(args.len());
                for arg in args {
                    inner_args.push(self.get_register(arg)?);
                }
                let v = f(inner_args);
                self.insert_register(output, v)
            },
            Instruction::CallExtensionModule {
                module,
                function,
                this,
                args,
                output,
            } => {
               let m = match self.modules.get(module) {
                    None => {
                        return Err(VMError::InvalidModule(format!(
                            "Module {} does not exist",
                            module
                        )))
                    }
                    Some(m) => {
                        m.clone()
                    }
                };
                let this = self.get_register(this)?;
                let rigz_type = this.rigz_type();
                let f = match m.extension_functions.get(&rigz_type) {
                    None => match m.extension_functions.get(&RigzType::Any) {
                        None => {
                            return Err(VMError::InvalidModuleFunction(format!(
                                "Module {}.{:?} does not exist (Any does not exist)",
                                module, rigz_type
                            )))
                        }
                        Some(def) => match def.get(function) {
                            None => {
                                return Err(VMError::InvalidModuleFunction(format!(
                                    "Module extension {}.{} does not exist",
                                    module, function
                                )))
                            }
                            Some(f) => {
                                f.clone()
                            }
                        },
                    },
                    Some(def) => match def.get(function) {
                        None => {
                            return Err(VMError::InvalidModuleFunction(format!(
                                "Module extension {}.{} does not exist",
                                module, function
                            )))
                        }
                        Some(f) => {
                            f.clone()
                        }
                    },
                };
                let mut inner_args = Vec::with_capacity(args.len());
                for arg in args {
                    inner_args.push(self.get_register(arg)?);
                }
                let v = f(this, inner_args);
                self.insert_register(output, v)
            },
        };
        Ok(VMState::Running)
    }

    fn current_scope(&mut self) -> Result<Scope<'vm>, VMError> {
        let scope_id = self.current.scope_id;
        match self.scopes.get(scope_id) {
            None => Err(VMError::ScopeError(format!(
                "Scope {} does not exist",
                scope_id
            ))),
            Some(s) => Ok(s.clone()),
        }
    }

    pub fn run(&mut self) -> Result<Value<'vm>, VMError> {
        loop {
            let scope = self.current_scope()?;
            let len = scope.instructions.len();
            if self.current.pc >= len {
                // TODO this should probably be an error requiring explicit halt, halt 0 returns none
                break;
            }

            let instruction = self.current.next_instruction(&scope);
            match self.process_instruction(instruction)? {
                VMState::Running => {}
                VMState::Done(v) => return Ok(v)
            };
        }
        Ok(Value::None)
    }

    pub fn load_mut(&mut self, name: String, reg: Register) -> Result<(), VMError> {
        match self.current.variables.entry(name) {
            Entry::Occupied(mut var) => match var.get() {
                Variable::Let(_) => {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Cannot overwrite let variable: {}",
                        *var.key()
                    )))
                }
                Variable::Mut(_) => {
                    var.insert(Variable::Mut(reg));
                }
            },
            Entry::Vacant(e) => {
                e.insert(Variable::Mut(reg));
            }
        }
        Ok(())
    }

    pub fn load_let(&mut self, name: String, reg: Register) -> Result<(), VMError> {
        match self.current.variables.entry(name) {
            Entry::Occupied(v) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Cannot overwrite let variable: {}",
                    *v.key()
                )))
            }
            Entry::Vacant(e) => {
                e.insert(Variable::Let(reg));
            }
        }
        Ok(())
    }

    pub fn call_frame(&mut self, scope_index: usize, output: Register) -> Result<(), VMError> {
        if self.scopes.len() <= scope_index {
            return Err(VMError::ScopeDoesNotExist(format!(
                "{} does not exist",
                scope_index
            )));
        }
        let current = std::mem::take(&mut self.current);
        self.frames.push(current);
        self.current = CallFrame::child(scope_index, self.frames.len() - 1, output);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::number::Number;
    use crate::number::Number::Int;
    use crate::value::Value;
    use crate::{Module, RigzType, VMBuilder};
    use indexmap::IndexMap;
    use std::str::FromStr;

    #[test]
    fn value_eq() {
        assert_eq!(Value::None, Value::None);
        assert_eq!(Value::None, Value::Bool(false));
        assert_eq!(Value::None, Value::Number(Number::Int(0)));
        assert_eq!(Value::None, Value::Number(Number::Float(0.0)));
        assert_eq!(Value::None, Value::Number(Number::UInt(0)));
        assert_eq!(Value::None, Value::String(String::new()));
        assert_eq!(Value::Bool(false), Value::String(String::new()));
        assert_eq!(Value::Number(Number::UInt(0)), Value::String(String::new()));
    }

    #[test]
    fn load_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::Number(Number::Int(42))
        );
    }

    #[test]
    fn cast_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_cast_instruction(4, RigzType::String, 7)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&7).unwrap().clone(),
            Value::String(42.to_string())
        );
    }

    #[test]
    fn add_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_copy_instruction(4, 37)
            .add_add_instruction(4, 37, 82)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&82).unwrap().clone(),
            Value::Number(Number::Int(84))
        );
    }

    #[test]
    fn copy_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_copy_instruction(4, 37)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&37).unwrap().clone(),
            Value::Number(Number::Int(42))
        );
    }

    #[test]
    fn shr_works_str_number() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .add_load_instruction(3, Value::Number(Number::Int(1)))
            .add_shr_instruction(2, 3, 4)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::String(String::from_str("ab").unwrap())
        );
    }

    #[test]
    fn shl_works_str_number() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .add_load_instruction(3, Value::Number(Number::Int(1)))
            .add_shl_instruction(2, 3, 4)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::String(String::from_str("bc").unwrap())
        );
    }

    #[test]
    fn call_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .enter_scope()
            .add_copy_instruction(2, 3)
            .exit_scope(3)
            .add_call_instruction(1, 3)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&3).unwrap().clone(),
            Value::String(String::from_str("abc").unwrap())
        );
    }

    #[test]
    fn module_works<'vm>() {
        let mut builder = VMBuilder::new();
        fn hello(args: Vec<Value>) -> Value {
            println!("{}", Value::List(args));
            Value::None
        }
        let mut functions: IndexMap<&'vm str, fn(Vec<Value<'vm>>) -> Value<'vm>> = IndexMap::new();
        functions.insert("hello", hello);

        let module = Module {
            name: "test",
            functions,
            extension_functions: Default::default(),
        };
        let mut vm = builder
            .register_module(module)
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .add_call_module_instruction("test", "hello", vec![2], 3)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&3).unwrap().clone(), Value::None);
    }
}
