pub mod child_of;

use std::fmt::Debug;

use crate::embed::{EntryTokens, GenerateContext, IndexTokens};

pub trait MarkerTrait: Send + Sync + Debug {
    fn implementation(
        &self,
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> proc_macro2::TokenStream;
}
