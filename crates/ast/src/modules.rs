use crate::program::ObjectDefinition;
use crate::{ModuleTraitDefinition, Parser};
use rigz_core::{Definition, Dependency, Module, Object};

pub struct ParsedDependency {
    pub dependency: Dependency,
    pub object_definition: ObjectDefinition,
}

impl ParsedDependency {
    pub fn new<T: ParsedObject>() -> Self {
        Self {
            dependency: Dependency::new::<T>(),
            object_definition: T::object_definition(),
        }
    }
}

pub trait ParsedModule: Module + Send + Sync {
    fn parsed_dependencies() -> Vec<ParsedDependency>
    where
        Self: Sized,
    {
        vec![]
    }

    fn module_definition() -> ModuleTraitDefinition
    where
        Self: Sized,
    {
        let mut parser = match Parser::prepare(Self::trait_definition(), false) {
            Ok(p) => p,
            Err(e) => panic!("Failed to setup parser {} - {e}", Self::name()),
        };
        match parser.parse_module_trait_definition() {
            Ok(m) => m,
            Err(e) => panic!("Failed to parse {} - {e}", Self::name()),
        }
    }
}

pub trait ParsedObject: Object {
    fn object_definition() -> ObjectDefinition
    where
        Self: Sized,
    {
        let mut parser = match Parser::prepare(Self::trait_definition(), false) {
            Ok(p) => p,
            Err(e) => panic!("Failed to setup parser {} - {e}", Self::name()),
        };
        match parser.parse_object_definition() {
            Ok(m) => m,
            Err(e) => panic!("Failed to parse {} - {e}", Self::name()),
        }
    }
}
