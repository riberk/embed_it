use sha2::{Sha224, Sha256, Sha384, Sha512};
use syn::parse_quote;

use super::{HashTrait, digest::DigestHashAlg};

pub const SHA2_224: &HashTrait<DigestHashAlg<Sha224>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA2_224.id,
    || parse_quote!(::embed_it::Sha2_224Hash),
    || parse_quote!(sha2_224),
));

pub const SHA2_256: &HashTrait<DigestHashAlg<Sha256>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA2_256.id,
    || parse_quote!(::embed_it::Sha2_256Hash),
    || parse_quote!(sha2_256),
));

pub const SHA2_384: &HashTrait<DigestHashAlg<Sha384>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA2_384.id,
    || parse_quote!(::embed_it::Sha2_384Hash),
    || parse_quote!(sha2_384),
));

pub const SHA2_512: &HashTrait<DigestHashAlg<Sha512>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA2_512.id,
    || parse_quote!(::embed_it::Sha2_512Hash),
    || parse_quote!(sha2_512),
));

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::super::{HashAlg, Hasher};

    #[test]
    fn check() {
        let mut hasher = super::SHA2_224.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("ea09ae9cc6768c50fcee903ed054556e5bfc8347907f12598aa24193")
        );

        let mut hasher = super::SHA2_256.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
        );

        let mut hasher = super::SHA2_384.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!(
                "59e1748777448c69de6b800d7a33bbfb9ff1b463e44354c3553bcdb9c666fa90125a3c79f90397bdf5f6a13de828684f"
            )
        );

        let mut hasher = super::SHA2_512.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!(
                "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
            )
        );
    }
}
