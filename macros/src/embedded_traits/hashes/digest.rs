use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use digest::Digest;

use super::HashAlg;

#[derive(Debug)]
pub struct DigestHashAlg<D> {
    _p: PhantomData<fn() -> D>,
    id: String,
    trait_path: syn::Path,
    trait_method: syn::Ident,
}

impl<D: Digest + std::io::Write> DigestHashAlg<D> {
    pub fn new<Id: Into<Cow<'static, str>>>(
        id: Id,
        trait_path: syn::Path,
        trait_method: syn::Ident,
    ) -> Self {
        Self {
            _p: PhantomData,
            id: id.into().into_owned(),
            trait_path,
            trait_method,
        }
    }
    
    pub fn id(&self) -> &str {
        &self.id
    }
    
    pub fn trait_path(&self) -> &syn::Path {
        &self.trait_path
    }
    
    pub fn trait_method(&self) -> &syn::Ident {
        &self.trait_method
    }
}

struct DigestHasher<H>(H);

impl<H: std::io::Write> std::io::Write for DigestHasher<H> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl<H: Digest + std::io::Write + 'static> super::Hasher for DigestHasher<H> {
    fn hash(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl<D: Digest + std::io::Write + 'static> HashAlg for DigestHashAlg<D> {
    fn id(&self) -> &str {
        &self.id
    }

    fn trait_path(&self) -> syn::Path {
        (self.trait_path)()
    }

    fn trait_method(&self) -> syn::Ident {
        (self.trait_method)()
    }

    fn hash_len(&self) -> usize {
        <D as Digest>::output_size()
    }

    fn make_hasher(&self) -> impl super::Hasher + 'static {
        DigestHasher(D::new())
    }
}
