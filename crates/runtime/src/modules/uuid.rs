use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use log::warn;
use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use uuid::Uuid;

derive_object! {
    "UUID",
    struct UUID {
        uuid: Uuid
    },
    r#"object UUID
        Self(value: String? = none)

        fn Self.to_s -> String
    end"#
}

impl From<Uuid> for UUID {
    fn from(value: Uuid) -> Self {
        Self {
            uuid: value
        }
    }
}

impl AsPrimitive<ObjectValue, Rc<RefCell<ObjectValue>>> for UUID {}

impl UUIDObject for UUID {
    fn to_s(&self) -> String {
        self.uuid.to_string()
    }
}

impl CreateObject for UUID {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized
    {
        if args.is_empty() {
            return Ok(UUID::default())
        }

        if args.len() > 1 {
            warn!("Ignoring additional args for UUID - {:?}", &args.0[1..]);
        }

        let first = args.0[0].borrow();
        if let ObjectValue::Primitive(PrimitiveValue::String(s)) = first.deref() {
            match Uuid::parse_str(s) {
                Ok(u) => Ok(u.into()),
                Err(e) => Err(VMError::runtime(format!("Cannot create UUID from {s}, {e:?}")))
            }
        } else {
            Err(VMError::UnsupportedOperation(format!("Cannot create UUID from {first}")))
        }
    }
}

derive_module! {
    [UUID],
    r#"
trait UUID
    fn v4 -> UUID::UUID

    fn create(input: String) -> UUID::UUID!
        UUID::UUID.new input
    end
end
"#
}

impl RigzUUID for UUIDModule {
    fn v4(&self) -> ObjectValue {
        ObjectValue::Object(Box::new(Into::<UUID>::into(Uuid::new_v4())))
    }
}
