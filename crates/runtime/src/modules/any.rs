use itertools::Itertools;
use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_vm::{out, outln};
use std::cell::RefCell;
use std::rc::Rc;

derive_module! {
    r#"
    import trait Any
        fn Any.clone -> Any
        fn Any.is_err -> Bool
        fn Any.is_none -> Bool
        fn Any.is_some -> Bool
        fn Any.is(type: Type) -> Bool
        fn Any.is_int -> Bool
        fn Any.is_float -> Bool
        fn Any.is_num -> Bool
        fn Any.to_b -> Bool
        fn Any.to_i -> Int!
        fn Any.to_f -> Float!
        fn Any.to_n -> Number!
        fn Any.to_s -> String
        fn Any.to_list -> List!
        fn Any.to_map -> Map!
        fn Any.type -> String
        fn Any.get(index) -> Any!?

        fn format(template: String, var args) -> String
        fn print(var args) -> None
        fn printf(template: String, var args) -> None
    end
"#
}

fn is_float(s: &str) -> bool {
    let mut float = false;
    for c in s.chars() {
        if c == '.' {
            if float {
                float = false;
                break;
            }
            float = true;
        } else if !c.is_ascii_digit() {
            break;
        }
    }
    float
}

impl RigzAny for AnyModule {
    fn any_clone(&self, this: ObjectValue) -> ObjectValue {
        this.clone()
    }

    fn any_is_err(&self, this: ObjectValue) -> bool {
        matches!(this, ObjectValue::Primitive(PrimitiveValue::Error(_)))
    }

    fn any_is_none(&self, this: ObjectValue) -> bool {
        // todo should error be counted as none?
        matches!(this, ObjectValue::Primitive(PrimitiveValue::None))
    }

    fn any_is_some(&self, this: ObjectValue) -> bool {
        // todo should error count as some?
        !matches!(this, ObjectValue::Primitive(PrimitiveValue::None))
    }

    fn any_is(&self, this: ObjectValue, rigz_type: RigzType) -> bool {
        this.rigz_type() == rigz_type
    }

    fn any_is_int(&self, this: ObjectValue) -> bool {
        match this {
            ObjectValue::Primitive(p) => match p {
                PrimitiveValue::Number(Number::Int(_)) => true,
                PrimitiveValue::String(s) => s.trim().chars().all(|c| c.is_ascii_digit()),
                _ => false,
            },
            _ => false,
        }
    }

    fn any_is_float(&self, this: ObjectValue) -> bool {
        match this {
            ObjectValue::Primitive(p) => match p {
                PrimitiveValue::Number(Number::Float(_)) => true,
                PrimitiveValue::String(s) => is_float(s.trim()),
                _ => false,
            },
            _ => false,
        }
    }

    fn any_is_num(&self, this: ObjectValue) -> bool {
        match this {
            ObjectValue::Primitive(p) => match p {
                PrimitiveValue::Number(_) => true,
                PrimitiveValue::String(s) => {
                    let s = s.trim();
                    s.chars().all(|c| c.is_ascii_digit()) || is_float(s)
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn any_to_b(&self, this: ObjectValue) -> bool {
        this.to_bool()
    }

    fn any_to_i(&self, this: ObjectValue) -> Result<i64, VMError> {
        this.to_int()
    }

    fn any_to_f(&self, this: ObjectValue) -> Result<f64, VMError> {
        this.to_float()
    }

    fn any_to_n(&self, this: ObjectValue) -> Result<Number, VMError> {
        this.to_number()
    }

    fn any_to_s(&self, this: ObjectValue) -> String {
        this.to_string()
    }

    fn any_to_list(&self, this: ObjectValue) -> Result<Vec<ObjectValue>, VMError> {
        this.to_list()
    }

    fn any_to_map(&self, this: ObjectValue) -> Result<IndexMap<ObjectValue, ObjectValue>, VMError> {
        this.to_map()
    }

    fn any_type(&self, this: ObjectValue) -> String {
        this.rigz_type().to_string()
    }

    fn any_get(
        &self,
        this: ObjectValue,
        index: ObjectValue,
    ) -> Result<Option<ObjectValue>, VMError> {
        this.get(&index)
    }

    fn format(&self, template: String, args: Vec<ObjectValue>) -> String {
        let mut res = template;
        for arg in args {
            let l = arg.to_string();
            res = res.replacen("{}", l.as_str(), 1);
        }
        res
    }

    fn print(&self, args: Vec<ObjectValue>) {
        let s = args.iter().map(|a| a.to_string()).join("");
        out!("{s}")
    }

    fn printf(&self, template: String, args: Vec<ObjectValue>) {
        outln!("{}", self.format(template, args))
    }
}
