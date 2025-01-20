use std::{collections::HashMap, fmt::Debug, fs::OpenOptions, io::BufReader};

use quote::quote;

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

#[cfg(feature = "md5")]
pub mod md5;

#[cfg(feature = "sha1")]
pub mod sha1;

#[cfg(feature = "sha2")]
pub mod sha2;

#[cfg(feature = "sha3")]
pub mod sha3;

#[cfg(feature = "blake3")]
pub mod blake3;

#[cfg(feature = "digest")]
pub mod digest;

pub mod ids;

pub trait HashAlg: Send + Sync {
    fn id(&self) -> &'static str;
    fn trait_path(&self) -> syn::Path;
    fn trait_method(&self) -> syn::Ident;
    fn hash_len(&self) -> usize;
    fn make_hasher(&self) -> impl Hasher;
}

pub trait Hasher: std::io::Write {
    fn hash(&mut self, data: &[u8]);
    fn finalize(self) -> Vec<u8>;
}

#[derive(Debug)]
pub struct HashTrait<T>(T);

impl<T: HashAlg + Debug> HashTrait<T> {
    pub const fn new(alg: T) -> Self {
        Self(alg)
    }
}

#[derive(Debug, Default)]
struct Hashes(HashMap<&'static str, Vec<u8>>);

impl<T: HashAlg + Debug> EmbeddedTrait for HashTrait<T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }

    fn path(&self, _: usize) -> syn::Path {
        self.0.trait_path()
    }

    fn definition(&self, _: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn impl_body(
        &self,
        ctx: &mut crate::embed::GenerateContext<'_>,
        entries: &[crate::embed::EntryTokens],
        _index: &[crate::embed::IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let hash = match &ctx.entry {
            crate::fs::Entry::Dir(_) => {
                let mut hasher = self.0.make_hasher();
                for entry in entries {
                    let name = &entry.entry.path().file_name;
                    let entry_hash = entry.items.get::<Hashes>().and_then(|h| h.0.get(self.id()));
                    hasher.hash(name.as_bytes());

                    if let Some(entry_hash) = entry_hash {
                        hasher.hash(entry_hash);
                    }
                }

                hasher.finalize()
            }
            crate::fs::Entry::File(info) => {
                let file_path = info.path().origin_path();
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
                let mut hasher = self.0.make_hasher();
                std::io::copy(&mut reader, &mut hasher).map_err(|e| {
                    MakeEmbeddedTraitImplementationError::with_error(
                        format!("Unable to hash content of {file_path:?}"),
                        e,
                    )
                })?;
                hasher.finalize()
            }
        };
        let hash_len = hash.len();
        let method = self.0.trait_method();
        let res = quote! {
            fn #method(&self) -> &'static [u8; #hash_len] {
                const VALUE: &[u8; #hash_len] = &[#(#hash),*];
                VALUE
            }
        };

        ctx.items
            .get_or_default::<Hashes>()
            .0
            .insert(self.id(), hash);
        Ok(res)
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        let method = self.0.trait_method();
        let hash_len = self.0.hash_len();
        quote! {
            fn #method(&self) -> &'static [u8; #hash_len] {
                match self {
                    Self::Dir(d) => d.#method(),
                    Self::File(f) => f.#method(),
                }
            }
        }
    }
}
