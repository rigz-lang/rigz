use crate::{ModuleTraitDefinition, Parser};
use rigz_core::{Definition, Module, Object};

pub trait ParsedModule: Module + Send + Sync {
    fn module_definition(&self) -> ModuleTraitDefinition {
        let mut parser = match Parser::prepare(self.trait_definition(), false) {
            Ok(p) => p,
            Err(e) => panic!("Failed to setup parser {} - {e}", self.name()),
        };
        match parser.parse_module_trait_definition() {
            Ok(m) => m,
            Err(e) => panic!("Failed to parse {} - {e}", self.name()),
        }
    }
}

pub trait ParsedObject: Object {
    fn object_definition(&self) -> ModuleTraitDefinition {
        let mut parser = match Parser::prepare(self.trait_definition(), false) {
            Ok(p) => p,
            Err(e) => panic!("Failed to setup parser {} - {e}", self.name()),
        };
        match parser.parse_module_trait_definition() {
            Ok(m) => m,
            Err(e) => panic!("Failed to parse {} - {e}", self.name()),
        }
    }
}
