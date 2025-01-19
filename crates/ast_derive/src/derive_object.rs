use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use rigz_ast::Parser;
use rigz_core::derive::Tokens;
use rigz_core::RigzType;
use syn::parse::{Parse, ParseStream};
use syn::{token, ItemStruct, LitStr, Token, Type, Visibility};

enum ObjectDefinition {
    Ident(Ident),
    Struct(ItemStruct),
}

pub(crate) struct DeriveObject {
    parent: LitStr,
    definition: ObjectDefinition,
    literal: LitStr,
    display: bool,
}

impl Parse for DeriveObject {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent = input.parse()?;
        input.parse::<Token![,]>()?;
        let definition = if input.peek(token::Struct) {
            ObjectDefinition::Struct(input.parse()?)
        } else {
            ObjectDefinition::Ident(input.parse()?)
        };
        input.parse::<Token![,]>()?;
        Ok(DeriveObject {
            parent,
            definition,
            literal: input.parse()?,
            display: true,
        })
    }
}

impl ToTokens for DeriveObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.full_definition())
    }
}

fn type_to_rigz_type(rust_type: &Type) -> RigzType {
    match rust_type {
        Type::Path(t) => {
            if t.qself.is_none() {
                if let Some(last) = t.path.segments.last() {
                    match last.ident.to_string().as_str() {
                        "i8" | "i16" | "i32" | "i64" => return RigzType::Int,
                        "f32" | "f64" => return RigzType::Float,
                        "bool" => return RigzType::Bool,
                        "String" => return RigzType::String,
                        _ => {}
                    }
                }
            }
        }
        t => {}
    }
    RigzType::Any
}

impl DeriveObject {
    fn full_definition(&self) -> Tokens {
        let parent = &self.parent;
        let (id, base, lit) = match &self.definition {
            ObjectDefinition::Ident(i) => (i, quote! {}, self.literal.value()),
            ObjectDefinition::Struct(s) => {
                let id = &s.ident;
                let pub_fields = s
                    .fields
                    .iter()
                    .filter(|f| matches!(f.vis, Visibility::Public(_)))
                    .map(|f| {
                        let name = f
                            .ident
                            .as_ref()
                            .expect("Anonymous Fields are not allowed in generated structs")
                            .to_string();
                        let rt = type_to_rigz_type(&f.ty);
                        (name, rt)
                    });
                let fields = pub_fields
                    .clone()
                    .map(|(name, rt)| quote! { (#name.to_string(), #rt) });
                let type_info = quote! {
                    impl rigz_core::WithTypeInfo for #id {
                        fn rigz_type(&self) -> rigz_core::RigzType {
                            rigz_core::RigzType::Custom(rigz_core::CustomType {
                                name: Self::name().to_string(),
                                fields: vec![#(#fields, )*],
                            })
                        }
                    }
                };

                let mut lit = self.literal.value();
                let name = id.to_string();
                let pub_fields = pub_fields
                    .map(|(name, rt)| {
                        if matches!(rt, RigzType::Any) {
                            format!("attr {name}")
                        } else {
                            format!("attr {name}, {}", rt)
                        }
                    })
                    .fold(String::new(), |mut res, next| {
                        res.push_str(next.as_str());
                        res.push('\n');
                        res
                    });
                lit = lit.replace(
                    format!("object {}", &name).as_str(),
                    format!("object {}::{}\n{}", parent.value(), &name, pub_fields).as_str(),
                );

                (
                    id,
                    quote! {
                        #[derive(derivative::Derivative)]
                        #[derive(Clone)]
                        #[derivative(Debug, Default, Hash, PartialOrd, PartialEq)]
                        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                        pub #s

                        #type_info
                    },
                    lit,
                )
            }
        };

        let base = if self.display {
            quote! {
                #base

                impl Display for #id {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{self:?}")
                    }
                }
            }
        } else {
            base
        };

        let name = id.to_string();
        let mut obj_def =
            Parser::prepare(lit.as_str(), false).expect("failed to setup object definition parser");
        let obj_def = obj_def
            .parse_object_definition()
            .expect("failed to parse object definition");
        let definition = quote! {
            impl rigz_core::Definition for #id {
                fn name() -> &'static str {
                    concat!(#parent, "::", #name)
                }

                fn trait_definition() -> &'static str {
                    #lit
                }
            }

            impl rigz_ast::ParsedObject for #id {
                fn object_definition() -> ObjectDefinition
                where
                    Self: Sized,
                {
                    #obj_def
                }
            }
        };

        let impl_object = impl_object(id, lit);

        quote! {
            #base

            #definition

            #impl_object
        }
    }
}

fn impl_object(name: &Ident, definition: String) -> Tokens {
    quote! {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl rigz_core::Object for #name {

        }
    }
}
