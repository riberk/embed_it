use std::{fmt::Debug, marker::PhantomData};

use digest::Digest;

use super::HashAlg;

// `fn` in order to be Send + Sync,
// because Ident is not, and PhantomData is not if `T`` is not
pub struct DigestHashAlg<D> {
    _p: PhantomData<fn() -> D>,
    id: &'static str,
    trait_path: fn() -> syn::Path,
    trait_method: fn() -> syn::Ident,
}

impl<D> Debug for DigestHashAlg<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DigestHashAlg")
            .field("id", &self.id)
            .field("trait_path", &(self.trait_path)())
            .field("trait_method", &(self.trait_method)())
            .finish()
    }
}

impl<D: Digest + std::io::Write> DigestHashAlg<D> {
    pub const fn new(
        id: &'static str,
        trait_path: fn() -> syn::Path,
        trait_method: fn() -> syn::Ident,
    ) -> Self {
        Self {
            _p: PhantomData,
            id,
            trait_path,
            trait_method,
        }
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

impl<H: Digest + std::io::Write> super::Hasher for DigestHasher<H> {
    fn hash(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl<D: Digest + std::io::Write> HashAlg for DigestHashAlg<D> {
    fn id(&self) -> &'static str {
        self.id
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

    fn make_hasher(&self) -> impl super::Hasher {
        DigestHasher(D::new())
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::DigestHashAlg;

    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "md5")]
    fn debug() {
        let alg = DigestHashAlg::<md5::Md5>::new(
            "Hash(custom)",
            || parse_quote!(Trait),
            || parse_quote!(custom_hash),
        );

        assert_eq!(
            &format!("{alg:?}"),
            r#"DigestHashAlg { id: "Hash(custom)", trait_path: Path { leading_colon: None, segments: [PathSegment { ident: Ident(Trait), arguments: PathArguments::None }] }, trait_method: Ident(custom_hash) }"#
        );
    }
}
