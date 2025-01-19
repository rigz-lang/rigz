use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use std::fmt::{Display, Formatter};

derive_object! {
    "Random",
    struct Random {
        pub seed: i64,
        stream: u64,
        offset: u128,
        #[cfg_attr(feature = "serde", serde(skip))]
        #[derivative(Hash="ignore", PartialEq="ignore", PartialOrd="ignore")]
        rng: InnerRng,
    },
    r#"object Random
        Self(seed: Number? = none)
        fn Self.offset -> (Number, Number)
        fn Self.stream -> (Number, Number)
        fn mut Self.set_offset(offset: (Number, Number))
        fn mut Self.set_stream(stream: Number)
        fn mut Self.set_seed(seed: Number)
        fn mut Self.next_int -> Int
        fn mut Self.next_float -> Float
        fn mut Self.next_bool(percent: Float = 0.5) -> Bool
    end
    "#
}

#[derive(Clone, Debug)]
struct InnerRng(ChaCha8Rng);

impl From<ChaCha8Rng> for InnerRng {
    fn from(value: ChaCha8Rng) -> Self {
        value.into()
    }
}

impl Default for InnerRng {
    fn default() -> Self {
        InnerRng(ChaCha8Rng::from_entropy())
    }
}

impl AsPrimitive<ObjectValue> for Random {}

impl CreateObject for Random {
    fn create(value: ObjectValue) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        let seed = match value {
            ObjectValue::Primitive(p) => match p {
                PrimitiveValue::None => rand::random(),
                PrimitiveValue::Number(n) => n.to_int(),
                PrimitiveValue::String(s) => match s.parse() {
                    Ok(i) => i,
                    Err(e) => {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Cannot create {} from {s} - {e}",
                            Self::name()
                        )))
                    }
                },
                p => {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Cannot create {} from {p}",
                        Self::name()
                    )))
                }
            },
            v => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Cannot create {} from {v}",
                    Self::name()
                )))
            }
        };
        let mut rng: InnerRng = ChaCha8Rng::seed_from_u64(seed as u64).into();
        Ok(Random {
            seed,
            offset: 0,
            stream: rng.0.get_stream(),
            rng,
        })
    }

    fn post_deserialize(&mut self) {
        let mut rng: InnerRng = ChaCha8Rng::seed_from_u64(self.seed as u64).into();
        rng.0.set_word_pos(self.offset);
        rng.0.set_stream(self.stream);
        self.rng = rng;
    }
}

derive_module! {
    [Random],
    r#"trait Random
        fn create(seed: Number? = none) -> Random::Random!
             Random::Random.new seed
        end

        fn next_int -> Int
        fn next_float -> Float
        fn next_bool(percent: Float = 0.5) -> Bool
    end"#
}

impl RigzRandom for RandomModule {
    fn next_int(&self) -> i64 {
        rand::random()
    }

    fn next_float(&self) -> f64 {
        rand::random()
    }

    fn next_bool(&self, percent: f64) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_bool(percent)
    }
}
