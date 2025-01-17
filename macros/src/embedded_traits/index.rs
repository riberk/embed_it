use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{bool_like_enum::BoolLikeEnum, EntryTokens, GenerateContext, IndexTokens},
    fs::EntryKind,
};

use super::EmbeddedTrait;

pub struct IndexTrait;

fn ident() -> syn::Ident {
    parse_quote!(Index)
}

fn method() -> syn::Ident {
    parse_quote!(get)
}

impl EmbeddedTrait for IndexTrait {
    fn path(&self, nesting: usize) -> syn::Path {
        GenerateContext::make_nested_path(nesting, ident())
    }

    fn impl_body(
        &self,
        ctx: &GenerateContext<'_>,
        _entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
        if ctx.entry.kind() != EntryKind::Dir {
            panic!("Only dirs are supported to derive '{:?}'", ident())
        }
        let entry_path = &ctx.entry_path;
        let index_len = index.len();
        let index = index
            .iter()
            .fold(proc_macro2::TokenStream::new(), |mut acc, tokens| {
                let IndexTokens {
                    relative_path,
                    struct_path,
                    kind,
                    ..
                } = tokens;
                let kind_ident = kind.ident();
                acc.extend(quote! {
                    map.insert(#relative_path, #entry_path::#kind_ident(&#struct_path));
                });
                acc
            });
        let index = quote! {
            let mut map = ::std::collections::HashMap::with_capacity(#index_len);
            #index
            map
        };
        let method = method();
        let value_get = if ctx.settings.support_alt_separator.as_bool() {
            quote! {
                VALUE.get(path.replace("\\", "/").as_str())
            }
        } else {
            quote! {
                VALUE.get(path)
            }
        };

        quote! {
            fn #method(&self, path: &str) -> Option<&'static #entry_path> {
                static VALUE: ::std::sync::LazyLock<::std::collections::HashMap<&'static str, #entry_path>> = ::std::sync::LazyLock::new(|| {
                    #index
                });
                #value_get
            }
        }
    }

    fn definition(&self, entry_path: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        let ident = ident();
        let method = method();
        Some(quote! {
            pub trait #ident {
                fn #method(&self, path: &str) -> Option<&'static #entry_path>;
            }
        })
    }

    fn id(&self) -> &'static str {
        "Index"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        panic!("Only dirs are supported to derive '{:?}'", ident())
    }
}
