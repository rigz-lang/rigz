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
mod vm;

use crate::prepare::ProgramParser;
use crate::RigzBuilder;

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
pub use vm::VMModule;

impl<T: RigzBuilder> ProgramParser<'_, T> {
    pub fn add_default_modules(&mut self) {
        self.register_module(VMModule);
        self.register_module(AnyModule);
        self.register_module(AssertionsModule);
        self.register_module(NumberModule);
        self.register_module(StringModule);
        self.register_module(CollectionsModule);
        self.register_module(LogModule);
        self.register_module(JSONModule);
        self.register_module(FileModule);
        self.register_module(DateModule);
        self.register_module(UUIDModule);
        self.register_module(RandomModule);
        self.register_module(MathModule);
    }
}
