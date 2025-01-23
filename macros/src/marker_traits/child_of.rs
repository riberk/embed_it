use proc_macro2::TokenStream;
use quote::quote;
use syn::PathSegment;

use crate::embed::{nested_module_path, EntryTokens, GenerateContext, IndexTokens};

use super::MarkerTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChildOfMarker;

impl MarkerTrait for ChildOfMarker {
    fn implementation(
        &self,
        ctx: &mut GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
        let struct_ident = &ctx.struct_ident;

        let mut stream = TokenStream::new();
        for (level, parent) in ctx.parents.iter().rev().enumerate() {
            let mut parent_path = syn::Path {
                leading_colon: None,
                segments: nested_module_path(level + 1),
            };

            parent_path.segments.push(PathSegment {
                ident: parent.struct_ident.clone(),
                arguments: syn::PathArguments::None,
            });

            stream.extend(quote! {
                #[automatically_derived]
                impl ::embed_it::ChildOf<#parent_path, #level> for #struct_ident {}
            });
        }

        stream
    }
}
