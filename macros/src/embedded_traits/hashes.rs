use std::{collections::HashMap, fmt::Debug, fs::OpenOptions, io::BufReader};

use quote::quote;

use super::{enabled_trait::EnabledTrait, EmbeddedTrait, MakeEmbeddedTraitImplementationError};
use super::feature_disabled::FeatureDisabled;

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

pub struct HashTraitFactory;

macro_rules! hash_factory {
    ($feature: literal, $err: ident, $name: ident, $create: expr) => {
        #[cfg(feature = $feature)]
        pub fn $name(&self) -> Result<EnabledTrait, FeatureDisabled> {
            Ok($create)
    
        }
    
        #[cfg(not(feature = $feature))]
        pub fn $name(&self) -> Result<EnabledTrait, FeatureDisabled> {
            Err(FeatureDisabled::$err())
        }    
    };
}

impl HashTraitFactory {
    hash_factory!("md5", md5, md5, md5::md5_trait());
    
    hash_factory!("sha1", sha1, sha1, sha1::sha1_trait());

    hash_factory!("sha2", sha2, sha2_224, sha2::sha2_224_trait());
    hash_factory!("sha2", sha2, sha2_256, sha2::sha2_256_trait());
    hash_factory!("sha2", sha2, sha2_384, sha2::sha2_384_trait());
    hash_factory!("sha2", sha2, sha2_512, sha2::sha2_512_trait());

    hash_factory!("sha3", sha3, sha3_224, sha3::sha3_224_trait());
    hash_factory!("sha3", sha3, sha3_256, sha3::sha3_256_trait());
    hash_factory!("sha3", sha3, sha3_384, sha3::sha3_384_trait());
    hash_factory!("sha3", sha3, sha3_512, sha3::sha3_512_trait());

    #[cfg(feature = "blake3")]
    pub fn blake3(&self, len: usize) -> Result<EnabledTrait, FeatureDisabled> {
        blake3::blake3_trait(len)
    }

    #[cfg(not(feature = "blake3"))]
    pub fn blake3(&self, len: usize) -> Result<EnabledTrait, FeatureDisabled> {
        Err(FeatureDisabled::blake3())
    }
}

pub trait HashAlg: 'static {
    fn id(&self) -> &str;
    fn trait_path(&self) -> syn::Path;
    fn trait_method(&self) -> syn::Ident;
    fn hash_len(&self) -> usize;
    fn make_hasher(&self) -> impl Hasher;
}

pub trait Hasher: std::io::Write + 'static {
    fn hash(&mut self, data: &[u8]);
    fn finalize(self) -> Vec<u8>;
}

#[derive(Debug)]
pub struct HashTrait<T> {
    alg: T,
    id: String,
}

impl<T: HashAlg + Debug> HashTrait<T> {
    #[cfg(any(
        feature = "md5",
        feature = "sha1",
        feature = "sha2",
        feature = "sha3",
        feature = "blake3"
    ))]
    pub fn new(alg: T) -> Self {
        Self {
            id: format!("Hash({})", alg.id()),
            alg,
        }
    }
}

#[derive(Debug, Default)]
struct Hashes(HashMap<String, Vec<u8>>);

impl<T: HashAlg + Debug> EmbeddedTrait for HashTrait<T> {
    fn id(&self) -> &str {
        &self.id
    }

    fn path(&self, _: usize) -> syn::Path {
        self.alg.trait_path()
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
        let mut hasher = self.alg.make_hasher();
        match &ctx.entry {
            crate::fs::Entry::Dir(_) => {
                for entry in entries {
                    let name = &entry.entry.path().file_name;
                    let entry_hash = entry.items.get::<Hashes>().and_then(|h| h.0.get(self.id()));
                    hasher.hash(name.as_bytes());

                    if let Some(entry_hash) = entry_hash {
                        hasher.hash(entry_hash);
                    }
                }

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
                std::io::copy(&mut reader, &mut hasher).map_err(|e| {
                    MakeEmbeddedTraitImplementationError::with_error(
                        format!("Unable to hash content of {file_path:?}"),
                        e,
                    )
                })?;
            }
        };
        let hash = hasher.finalize();
        let hash_len = hash.len();
        let method = self.alg.trait_method();
        let res = quote! {
            fn #method(&self) -> &'static [u8; #hash_len] {
                const VALUE: &[u8; #hash_len] = &[#(#hash),*];
                VALUE
            }
        };

        ctx.items
            .get_or_default::<Hashes>()
            .0
            .insert(self.id().to_owned(), hash);
        Ok(res)
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        let method = self.alg.trait_method();
        let hash_len = self.alg.hash_len();
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
