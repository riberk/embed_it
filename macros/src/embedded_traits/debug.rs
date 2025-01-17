use quote::{quote, ToTokens};
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens},
    fs::EntryKind,
};

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

#[derive(Debug)]
pub struct DebugTrait;

impl EmbeddedTrait for DebugTrait {
    fn path(&self, _nesting: usize) -> syn::Path {
        parse_quote!(::std::fmt::Debug)
    }

    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        match ctx.entry.kind() {
            EntryKind::Dir => {
                let fields = entries.iter().fold(quote! {}, |mut accum, entry| {
                    let field_name = &entry.field_name;
                    let field_ident = &entry.field_ident;
                    if ctx.is_trait_implemented_for(entry.entry.kind(), &DebugTrait) {
                        accum.extend(quote! {
                            debug.field(#field_name, &self.#field_ident());
                        });
                    } else {
                        let struct_name = entry.struct_path.to_token_stream().to_string();
                        accum.extend(quote! {
                            debug.field(#field_name, &#struct_name);
                        });
                    }
                    accum
                });
                Ok(debug(ctx, fields))
            }
            EntryKind::File => {
                let file_len = ctx.entry.metadata().len();
                let debug_content = format!("<{} bytes>", file_len);
                Ok(debug(
                    ctx,
                    quote! {
                        debug.field("content", &#debug_content);
                    },
                ))
            }
        }
    }

    fn definition(&self, _: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Debug"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        quote! {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Dir(d) => f.debug_tuple("Dir").field(d).finish(),
                    Self::File(d) => f.debug_tuple("File").field(d).finish(),
                }
            }
        }
    }
}

fn debug(ctx: &GenerateContext<'_>, fields: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let struct_name = &ctx.struct_name;

    quote! {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            let mut debug = f.debug_struct(#struct_name);
            #fields
            debug.finish()
        }
    }
}
