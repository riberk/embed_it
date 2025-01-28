use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Ident, Token, TypeParamBound};


use crate::{embed::{attributes::field::FieldTraits, EntryTokens, GenerateContext, IndexTokens}, marker_traits::MarkerTrait};

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

pub trait MainTrait {
    fn trait_ident(&self) -> &Ident;

    fn field_factory_trait_ident(&self) -> &Ident;

    /// Which traits must be implemented for any of implementors of that trait
    fn embedded_traits(&self) -> impl Iterator<Item = &dyn EmbeddedTrait>;

    fn fields(&self) -> &FieldTraits;

    fn markers(&self) -> impl Iterator<Item = &dyn MarkerTrait>;

    fn struct_impl(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
    ) -> proc_macro2::TokenStream;

    /// That trait implements debug
    fn is_trait_implemented(&self, expected: &impl EmbeddedTrait) -> bool {
        let expected = expected.id();
        self.embedded_traits().any(|t| t.id() == expected)
    }

    fn definition(&self) -> proc_macro2::TokenStream {
        let trait_ident = self.trait_ident();

        let mut bounds = Punctuated::<TypeParamBound, Token![+]>::new();
        bounds.push(parse_quote!(Send));
        bounds.push(parse_quote!(Sync));

        for t in self.embedded_traits() {
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
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let trait_ident = self.trait_ident();
        let mut impl_stream = quote! {};

        for t in self.embedded_traits() {
            impl_stream.extend(t.implementation(ctx, entries, index)?);
        }

        for m in self.markers() {
            impl_stream.extend(m.implementation(ctx, entries, index));
        }

        let trait_path = ctx.make_level_path(trait_ident.to_owned());
        let struct_impl = self.struct_impl(ctx, entries);
        let struct_ident = &ctx.struct_ident;
        impl_stream.extend(quote! {
            #struct_impl

            #[automatically_derived]
            impl #trait_path for #struct_ident {}
        });
        Ok(impl_stream)
    }
}
