use crate::derive::csv_vec;
use crate::{
    EventLifecycle, Lifecycle, MemoizedLifecycle, Stage, StatefulLifecycle, TestLifecycle,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl ToTokens for Lifecycle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Lifecycle::On(l) => quote! {
                Lifecycle::On(#l)
            },
            Lifecycle::After(l) => quote! {
                Lifecycle::After(#l)
            },
            Lifecycle::Memo(l) => quote! {
                Lifecycle::Memo(#l)
            },
            Lifecycle::Test(l) => quote! {
                Lifecycle::Test(#l)
            },
            Lifecycle::Composite(l) => {
                let csv = csv_vec(l);
                quote! {
                    Lifecycle::Composite(#csv)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for EventLifecycle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EventLifecycle { event } = self;
        tokens.extend(quote! {
            EventLifecycle {
                event: #event.into()
            }
        })
    }
}

impl ToTokens for MemoizedLifecycle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let MemoizedLifecycle { results } = self;
        let results: Vec<_> = results
            .into_iter()
            .map(|(k, v)| {
                let k = csv_vec(k);
                quote! {
                    (#k, #v),
                }
            })
            .collect();
        tokens.extend(quote! {
            MemoizedLifecycle {
                results: HashMap::from(#(#results)*)
            }
        });
    }
}

impl ToTokens for Stage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Stage::Parse => quote! { Stage::Parse },
            Stage::Run => quote! { Stage::Run },
            Stage::Halt => quote! { Stage::Halt },
            Stage::Custom(c) => quote! { Stage::Custom(#c.into()) },
        };
        tokens.extend(t)
    }
}

impl ToTokens for StatefulLifecycle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StatefulLifecycle { stage } = self;
        tokens.extend(quote! {
            StatefulLifecycle {
                stage: #stage,
            }
        })
    }
}

impl ToTokens for TestLifecycle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            TestLifecycle
        })
    }
}
