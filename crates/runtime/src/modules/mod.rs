mod any;
mod assertions;
mod collections;
mod date;
mod file;
mod html;
mod http;
mod json;
mod number;
mod random;
mod string;
mod uuid;
// mod vm;

use crate::prepare::{DependencyDefinition, ModuleDefinition, ProgramParser};
use std::sync::Arc;

pub use crate::modules::html::HtmlModule;
pub use crate::modules::http::HttpModule;
pub use any::AnyModule;
pub use assertions::AssertionsModule;
pub use collections::CollectionsModule;
pub use date::DateModule;
pub use file::FileModule;
pub use json::JSONModule;
pub use number::NumberModule;
pub use string::StringModule;
// pub use vm::VMModule;

pub use crate::modules::random::Random;
pub use crate::modules::uuid::UUID;
use rigz_ast::{ParsedDependency, ParsedModule, ParsedObject, ParserOptions, ValidationError};
use rigz_vm::VMBuilder;

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

    pub fn register_object<O: ParsedObject + 'static>(&mut self) -> Result<(), ValidationError> {
        let ParsedDependency {
            name,
            dependency,
            object_definition,
        } = ParsedDependency::new::<O>();
        let idx = self.builder.register_dependency(Arc::new(dependency));
        self.parsed_deps
            .insert(name, DependencyDefinition::Parsed(object_definition, idx));
        Ok(())
    }

    pub fn register_module<M: ParsedModule + 'static>(
        &mut self,
        module: M,
    ) -> Result<(), ValidationError> {
        let name = M::name();
        let def = M::module_definition();
        let deps = M::parsed_dependencies();
        let mut dep_names = Vec::with_capacity(deps.len());
        for ParsedDependency {
            name,
            dependency,
            object_definition,
        } in deps
        {
            let idx = self.builder.register_dependency(Arc::new(dependency));
            self.parsed_deps
                .insert(name, DependencyDefinition::Parsed(object_definition, idx));
            dep_names.push(name);
        }
        let index = self.builder.register_module(module);
        self.modules
            .insert(name, ModuleDefinition::Module(def, index, dep_names));
        Ok(())
    }

    pub fn add_default_modules(&mut self) -> Result<(), ValidationError> {
        // self.register_module(VMModule);
        self.register_module(AnyModule)?;
        self.register_module(AssertionsModule)?;
        self.register_module(NumberModule)?;
        self.register_module(StringModule)?;
        self.register_module(CollectionsModule)?;
        self.register_module(JSONModule)?;
        self.register_module(FileModule)?;
        self.register_module(DateModule)?;
        self.register_object::<UUID>()?;
        self.register_object::<Random>()?;
        self.register_module(HtmlModule)?; // http module depends on html
        self.register_module(HttpModule::default())?;
        Ok(())
    }
}
