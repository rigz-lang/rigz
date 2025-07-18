use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use std::ops::Deref;

derive_object! {
    "Random",
    struct Random {
        pub seed: i64,
        stream: u64,
        offset: u128,
        #[serde(skip)]
        #[derivative(Debug="ignore", Hash="ignore", PartialEq="ignore", PartialOrd="ignore")]
        rng: InnerRng,
    },
    r#"object Random
        Self(seed: Number? = none)
        fn Self.offset -> (Int, Int)
        fn Self.stream -> Int
        fn mut Self.set_offset(upper: Number, lower: Number)
        fn mut Self.set_stream(stream: Number)
        fn mut Self.set_seed(seed: Number)
        fn mut Self.next_int -> Int
        fn mut Self.next_float -> Float
        fn mut Self.next_bool(percent: Float = 0.5) -> Bool
    end
    "#
}

impl RandomObject for Random {
    fn offset(&self) -> (i64, i64) {
        let upper = (self.offset >> 64) as u64;
        let lower = self.offset as u64;
        (upper as i64, lower as i64)
    }

    fn stream(&self) -> i64 {
        self.stream as i64
    }

    fn mut_set_offset(&mut self, upper: Number, lower: Number) {
        let mut offset = lower.to_bits() as u128;
        offset |= (upper.to_bits() as u128) << 64;
        self.offset = offset;
    }

    fn mut_set_stream(&mut self, stream: Number) {
        self.stream = stream.to_bits();
    }

    fn mut_set_seed(&mut self, seed: Number) {
        self.seed = seed.to_int();
    }

    fn mut_next_int(&mut self) -> i64 {
        self.rng.0.next_u64() as i64
    }

    fn mut_next_float(&mut self) -> f64 {
        f64::from_bits(self.rng.0.next_u64())
    }

    fn mut_next_bool(&mut self, percent: f64) -> bool {
        self.rng.0.gen_bool(percent)
    }
}

#[derive(Clone)]
struct InnerRng(ChaCha8Rng);

impl From<ChaCha8Rng> for InnerRng {
    #[inline]
    fn from(value: ChaCha8Rng) -> Self {
        InnerRng(value)
    }
}

impl Default for InnerRng {
    #[inline]
    fn default() -> Self {
        ChaCha8Rng::from_entropy().into()
    }
}

impl AsPrimitive<ObjectValue> for Random {}

impl CreateObject for Random {
    fn create(value: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        let v = value.first()?;
        let seed = match v.borrow().deref() {
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
        let rng: InnerRng = ChaCha8Rng::seed_from_u64(seed as u64).into();
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
