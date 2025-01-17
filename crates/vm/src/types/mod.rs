use rigz_core::{Object, ObjectValue, RigzType};
use std::collections::HashMap;

enum TypeDefinition {
    Alias(String),
    Concrete(ConcreteDefinition),
}

struct TypeRegistry {
    types: HashMap<String, TypeDefinition>,
}

struct ConcreteDefinition {
    create: fn(ObjectValue) -> Box<dyn Object>,
    rigz_type: RigzType,
}
