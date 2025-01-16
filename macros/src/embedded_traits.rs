pub mod content;
pub mod debug;
pub mod entries;
pub mod index;
pub mod meta;
pub mod path;

use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Ident, Token, TraitBound, TypeParamBound};

use crate::embed::{EntryTokens, GenerateContext, IndexTokens};

pub static EMBEDED_TRAITS: LazyLock<HashMap<&'static str, &'static dyn EmbeddedTrait>> =
    LazyLock::new(|| {
        fn add(
            map: &mut HashMap<&'static str, &'static dyn EmbeddedTrait>,
            t: &'static dyn EmbeddedTrait,
        ) {
            let res = map.insert(t.id(), t);
            if res.is_some() {
                panic!("Duplicate trait id: {}", t.id())
            }
        }
        let mut map = HashMap::new();
        add(&mut map, &content::ContentTrait);
        add(&mut map, &debug::DebugTrait);
        add(&mut map, &entries::EntriesTrait);
        add(&mut map, &index::IndexTrait);
        add(&mut map, &meta::MetaTrait);
        add(&mut map, &path::PathTrait);
        map
    });

pub trait EmbeddedTrait: Send + Sync {
    fn id(&self) -> &'static str;

    fn path(&self, nesting: usize) -> syn::Path;

    /// Definition of the trait. If it is external trait (like Debug) it returns None
    fn definition(&self, entry_path: &syn::Ident) -> Option<proc_macro2::TokenStream>;

    fn bound(&self) -> TraitBound {
        let path = self.path(0);
        parse_quote!(#path)
    }

    fn impl_body(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> proc_macro2::TokenStream;

    fn implementation(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
        let trait_path = self.path(ctx.level);
        let struct_ident = &ctx.struct_ident;
        let body = self.impl_body(ctx, entries, index);

        quote! {
            #[automatically_derived]
            impl #trait_path for #struct_ident {
                #body
            }
        }
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream;
}

pub trait TraitAttr {
    /// The default name for that trait
    const DEFAULT_TRAIT_NAME: &str;
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str;

    /// User-defined trait name
    fn trait_name(&self) -> Option<&Ident>;

    /// User-defined field factory trait name
    fn field_factory_trait_name(&self) -> Option<&Ident>;

    /// Which traits must be implemented for any of implementors of that trait
    fn traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait>;

    fn struct_impl(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
    ) -> proc_macro2::TokenStream;

    /// That trait implements debug
    fn is_trait_implemented(&self, expected: &'static impl EmbeddedTrait) -> bool {
        let expected = expected.id();
        self.traits().any(|t| t.id() == expected)
    }

    fn trait_ident(&self) -> Cow<'_, Ident> {
        if let Some(name) = self.trait_name().as_ref() {
            Cow::Borrowed(name)
        } else {
            Cow::Owned(Ident::new(Self::DEFAULT_TRAIT_NAME, Span::call_site()))
        }
    }

    fn field_factory_trait_ident(&self) -> Cow<'_, Ident> {
        if let Some(name) = self.field_factory_trait_name().as_ref() {
            Cow::Borrowed(name)
        } else {
            Cow::Owned(Ident::new(
                Self::DEFAULT_FIELD_FACTORY_TRAIT_NAME,
                Span::call_site(),
            ))
        }
    }

    fn definition(&self) -> proc_macro2::TokenStream {
        let trait_ident = self.trait_ident();

        let mut bounds = Punctuated::<TypeParamBound, Token![+]>::new();
        bounds.push(parse_quote!(Send));
        bounds.push(parse_quote!(Sync));

        for t in self.traits() {
            bounds.push(TypeParamBound::Trait(t.bound()));
        }

        quote! {
            pub trait #trait_ident : #bounds {}
        }
    }

    /// Implements this trait (and its bounds) for an entry
    /// # Arguments
    ///
    /// * `self`
    /// * `ctx` - The context of generated entry
    /// * `entries` - Direct children entries
    /// * `index` - Recursive children including the direct
    fn implementation_stream(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
        let struct_ident = &ctx.struct_ident;
        let trait_ident = self.trait_ident();
        let mut impl_stream = quote! {};
        ctx.make_level_path(struct_ident.clone());
        for t in self.traits() {
            impl_stream.extend(t.implementation(ctx, entries, index));
        }

        let trait_path = ctx.make_level_path(trait_ident.into_owned());
        let struct_impl = self.struct_impl(ctx, entries);
        impl_stream.extend(quote! {
            #struct_impl

            #[automatically_derived]
            impl #trait_path for #struct_ident {}
        });
        impl_stream
    }
}
