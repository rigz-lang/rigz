mod any;
mod assertions;
mod collections;
mod date;
mod file;
mod html;
mod http;
mod json;
mod log;
mod math;
mod number;
mod random;
mod string;
mod uuid;
// mod vm;

use crate::prepare::{ModuleDefinition, ProgramParser};
use std::sync::Arc;

use crate::modules::html::HtmlModule;
use crate::modules::http::HttpModule;
pub use any::AnyModule;
pub use assertions::AssertionsModule;
pub use collections::CollectionsModule;
pub use date::DateModule;
pub use file::FileModule;
pub use json::JSONModule;
pub use log::LogModule;
pub use math::MathModule;
pub use number::NumberModule;
pub use random::RandomModule;
pub use string::StringModule;
pub use uuid::UUIDModule;
// pub use vm::VMModule;

use rigz_ast::{ParsedModule, ParserOptions, ValidationError};
use rigz_vm::{RigzBuilder, VMBuilder};

impl ProgramParser<'_, VMBuilder> {
    pub fn new() -> Self {
        let mut p = ProgramParser::default();
        p.add_default_modules()
            .expect("failed to register default modules");
        p
    }

    pub fn with_options(parser_options: ParserOptions) -> Self {
        let mut p = ProgramParser::default();
        p.parser_options = parser_options;
        p
    }

    pub fn register_module<M: ParsedModule + 'static>(
        &mut self,
        module: M,
    ) -> Result<(), ValidationError> {
        let name = M::name();
        let def = M::module_definition();
        for dep in M::parsed_dependencies() {
            let obj = dep.object_definition;
            let dep = self.builder.register_dependency(Arc::new(dep.dependency));
            self.parse_object_definition(obj, Some(dep))?;
        }
        let index = self.builder.register_module(module);
        self.modules
            .insert(name, ModuleDefinition::Module(def, index));
        Ok(())
    }

    pub fn add_default_modules(&mut self) -> Result<(), ValidationError> {
        // self.register_module(VMModule);
        self.register_module(AnyModule)?;
        self.register_module(AssertionsModule)?;
        self.register_module(NumberModule)?;
        self.register_module(StringModule)?;
        self.register_module(CollectionsModule)?;
        self.register_module(LogModule)?;
        self.register_module(JSONModule)?;
        self.register_module(FileModule)?;
        self.register_module(DateModule)?;
        self.register_module(UUIDModule)?;
        self.register_module(RandomModule)?;
        self.register_module(MathModule)?;
        self.register_module(HtmlModule)?; // http module depends on html
        self.register_module(HttpModule::default())?;
        Ok(())
    }
}
