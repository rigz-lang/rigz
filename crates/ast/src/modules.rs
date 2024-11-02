use crate::{ModuleTraitDefinition, Parser};
use rigz_vm::Module;

pub trait ParsedModule<'a>: Module<'a> {
    fn module_definition(&self) -> ModuleTraitDefinition<'static> {
        let mut parser = match Parser::prepare(self.trait_definition()) {
            Ok(p) => p,
            Err(e) => panic!("Failed to setup parser {} - {e}", self.name()),
        };
        match parser.parse_module_trait_definition() {
            Ok(m) => m,
            Err(e) => panic!("Failed to parse {} - {e}", self.name()),
        }
    }
}
