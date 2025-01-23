mod any;
mod assertions;
mod collections;
mod date;
mod file;
mod html;
mod http;
#[cfg(feature = "serde")]
mod json;
mod log;
mod math;
mod number;
mod random;
mod string;
mod uuid;
// mod vm;

use crate::prepare::ProgramParser;

use crate::modules::html::HtmlModule;
use crate::modules::http::HttpModule;
pub use any::AnyModule;
pub use assertions::AssertionsModule;
pub use collections::CollectionsModule;
pub use date::DateModule;
pub use file::FileModule;
#[cfg(feature = "serde")]
pub use json::JSONModule;
pub use log::LogModule;
pub use math::MathModule;
pub use number::NumberModule;
pub use random::RandomModule;
use rigz_ast::ValidationError;
use rigz_vm::RigzBuilder;
pub use string::StringModule;
pub use uuid::UUIDModule;
// pub use vm::VMModule;

impl<T: RigzBuilder> ProgramParser<'_, T> {
    pub fn add_default_modules(&mut self) -> Result<(), ValidationError> {
        // self.register_module(VMModule);
        self.register_module(AnyModule)?;
        self.register_module(AssertionsModule)?;
        self.register_module(NumberModule)?;
        self.register_module(StringModule)?;
        self.register_module(CollectionsModule)?;
        self.register_module(LogModule)?;
        #[cfg(feature = "serde")]
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
