use log::warn;
use rigz_ast::*;
use rigz_ast_derive::derive_object;
use rigz_core::*;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;
use uuid::Uuid;

derive_object! {
    pub struct UUID {
        uuid: Uuid
    },
    r#"object UUID
        Self(value: String? = none)

        fn Self.braced -> String
        fn Self.simple -> String
        fn Self.urn -> String

        fn random -> UUID
    end"#,
    skip_display
}

impl From<Uuid> for UUID {
    fn from(value: Uuid) -> Self {
        Self { uuid: value }
    }
}

impl AsPrimitive<ObjectValue, Rc<RefCell<ObjectValue>>> for UUID {}

impl Display for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl UUIDObject for UUID {
    fn braced(&self) -> String {
        self.uuid.braced().to_string()
    }
    fn simple(&self) -> String {
        self.uuid.simple().to_string()
    }
    fn urn(&self) -> String {
        self.uuid.urn().to_string()
    }

    fn static_random() -> ObjectValue
    where
        Self: Sized,
    {
        ObjectValue::Object(Box::new(Into::<UUID>::into(Uuid::new_v4())))
    }
}

impl CreateObject for UUID {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        if args.is_empty() {
            return Ok(UUID::default());
        }

        if args.len() > 1 {
            warn!("Ignoring additional args for UUID - {:?}", &args.0[1..]);
        }

        let first = args.0[0].borrow();
        if let ObjectValue::Primitive(PrimitiveValue::String(s)) = first.deref() {
            match Uuid::parse_str(s) {
                Ok(u) => Ok(u.into()),
                Err(e) => Err(VMError::runtime(format!(
                    "Cannot create UUID from {s}, {e:?}"
                ))),
            }
        } else {
            Err(VMError::UnsupportedOperation(format!(
                "Cannot create UUID from {first}"
            )))
        }
    }
}
