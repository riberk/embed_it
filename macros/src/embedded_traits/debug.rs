use embed_it_utils::entry::EntryKind;
use quote::{quote, ToTokens};
use syn::parse_quote;

use crate::embed::{
    attributes::embed::GenerationSettings, EntryTokens, GenerateContext, IndexTokens,
};

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

#[derive(Debug)]
pub struct DebugTrait;

impl EmbeddedTrait for DebugTrait {
    fn path(&self, _nesting: usize, _: &GenerationSettings) -> syn::Path {
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
                    let field_name = entry.field.name();
                    let field_ident = entry.field.ident();
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
                let file_len = ctx.entry.as_ref().value().metadata().len();
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

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Debug"
    }
}

fn debug(ctx: &GenerateContext<'_>, fields: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let struct_name = ctx.entry_struct_ident().name();

    quote! {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            let mut debug = f.debug_struct(#struct_name);
            #fields
            debug.finish()
        }
    }
}
