use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use chrono::{DateTime, Local, Utc};
use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;

macro_rules! date_common {
    ($tr: tt for $ty: ty) => {
        impl Display for $ty {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.date)
            }
        }

        impl AsPrimitive<ObjectValue, Rc<RefCell<ObjectValue>>> for $ty {
            fn to_int(&self) -> Result<i64, VMError> {
                Ok(self.date.timestamp_millis())
            }

            fn to_float(&self) -> Result<f64, VMError> {
                Ok(self.date.timestamp_millis() as f64)
            }
        }

        impl $tr for $ty {
            fn format(&self, template: String) -> String {
                format!("{}", self.date.format(&template))
            }
        }
    };
}

derive_object! {
    "Date",
    struct LocalDate {
        date: DateTime<Local>
    },
    r#"object LocalDate
        Self(value: Any? = none)

        fn Self.format(template: String) -> String
    end"#,
    skip_display
}

impl CreateObject for LocalDate {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized
    {
        if args.is_empty() || args.0[0].borrow().is_none() {
            return Ok(LocalDate {
                date: Local::now()
            })
        }

        // todo support from String & (Y, M, D, h, m, s, ms)
        if args.len() == 1 {
            let ms = args.0[0].borrow().to_int()?;
            if let Some(date) = DateTime::from_timestamp_millis(ms) {
                return Ok(LocalDate {
                    date: date.into()
                })
            }
        }

        Err(VMError::runtime(format!("Failed to create Date: {args:?}")))
    }
}

derive_object! {
    "Date",
    struct UTCDate {
        date: DateTime<Utc>
    },
    r#"object UTCDate
        Self(value: Any? = none)

        fn Self.format(template: String) -> String
    end"#,
    skip_display
}

impl CreateObject for UTCDate {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized
    {
        if args.is_empty() || args.0[0].borrow().is_none() {
            return Ok(Self {
                date: Utc::now()
            })
        }

        // todo support from String & (Y, M, D, h, m, s, ms)
        if args.len() == 1 {
            let ms = args.0[0].borrow().to_int()?;
            if let Some(date) = DateTime::from_timestamp_millis(ms) {
                return Ok(UTCDate {
                    date
                })
            }
        }

        Err(VMError::runtime(format!("Failed to create Date: {args:?}")))
    }
}

date_common!(LocalDateObject for LocalDate);
date_common!(UTCDateObject for UTCDate);

derive_module! {
    [LocalDate, UTCDate],
    r#"trait Date
    fn now -> Date::LocalDate = Date::LocalDate.new
    fn utc -> Date::UTCDate = Date::UTCDate.new
end"#
}

impl RigzDate for DateModule {}
