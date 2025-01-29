use std::{fmt::Debug, fs::OpenOptions, io::BufReader};

use embed_it_utils::entry::EntryKind;
use quote::quote;

use crate::embed::attributes::embed::GenerationSettings;

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

#[cfg(feature = "gzip")]
pub mod gzip;

#[cfg(feature = "zstd")]
pub mod zstd;

#[cfg(feature = "brotli")]
pub mod brotli;

pub mod ids;

pub trait CompressionAlg: Send + Sync {
    fn id(&self) -> &'static str;
    fn trait_path(&self) -> syn::Path;
    fn trait_method(&self) -> syn::Ident;
    fn make_compressor(&self) -> impl Compressor;
}

pub trait Compressor: std::io::Write {
    fn finalize(self) -> Result<Vec<u8>, FinalizeCompressorError>;
}

#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum FinalizeCompressorError {
    #[cfg(any(feature = "zstd", feature = "gzip"))]
    #[display("unable to finalize compression: {_0}")]
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct CompressionTrait<T>(T);

impl<T: CompressionAlg + Debug> CompressionTrait<T> {
    #[cfg(feature = "any-compression")]
    pub const fn new(alg: T) -> Self {
        Self(alg)
    }
}

impl<T: CompressionAlg + Debug> EmbeddedTrait for CompressionTrait<T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }

    fn path(&self, _: usize, _: &GenerationSettings) -> syn::Path {
        self.0.trait_path()
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn impl_body(
        &self,
        ctx: &mut crate::embed::GenerateContext<'_>,
        _entries: &[crate::embed::EntryTokens],
        _index: &[crate::embed::IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        if ctx.entry.kind() != EntryKind::File {
            return Err(MakeEmbeddedTraitImplementationError::UnsupportedEntry {
                entry: ctx.entry.kind(),
                trait_id: self.id(),
            });
        }
        let file_path = ctx.entry.as_ref().value().path().origin_path();
        let file = OpenOptions::new()
            .read(true)
            .create(false)
            .create_new(false)
            .append(false)
            .write(false)
            .truncate(false)
            .open(file_path)
            .map_err(|e| {
                MakeEmbeddedTraitImplementationError::with_error(
                    format!("Unable to open file {file_path:?}"),
                    e,
                )
            })?;
        let mut reader = BufReader::new(file);
        let mut compressor = self.0.make_compressor();

        std::io::copy(&mut reader, &mut compressor).map_err(|e| {
            MakeEmbeddedTraitImplementationError::with_error(
                format!(
                    "Unable to compress content of {file_path:?} with '{}'",
                    self.id()
                ),
                e,
            )
        })?;

        let content = compressor.finalize().map_err(|e| {
            MakeEmbeddedTraitImplementationError::with_error(
                format!("Unable to compress file {file_path:?} with {}", self.id()),
                e,
            )
        })?;
        let method = self.0.trait_method();
        let res = quote! {
            fn #method(&self) -> &'static [u8] {
                const VALUE: &[u8] = &[#(#content),*];
                VALUE
            }
        };

        Ok(res)
    }
}
