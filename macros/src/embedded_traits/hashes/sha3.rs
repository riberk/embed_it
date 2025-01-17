use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use syn::parse_quote;

use super::{digest::DigestHashAlg, HashTrait};

pub const SHA3_224: &HashTrait<DigestHashAlg<Sha3_224>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA3_224.id,
    || parse_quote!(::embed_it::Sha3_224Hash),
    || parse_quote!(sha3_224),
));

pub const SHA3_256: &HashTrait<DigestHashAlg<Sha3_256>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA3_256.id,
    || parse_quote!(::embed_it::Sha3_256Hash),
    || parse_quote!(sha3_256),
));

pub const SHA3_384: &HashTrait<DigestHashAlg<Sha3_384>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA3_384.id,
    || parse_quote!(::embed_it::Sha3_384Hash),
    || parse_quote!(sha3_384),
));

pub const SHA3_512: &HashTrait<DigestHashAlg<Sha3_512>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA3_512.id,
    || parse_quote!(::embed_it::Sha3_512Hash),
    || parse_quote!(sha3_512),
));

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::super::{HashAlg, Hasher};

    #[test]
    fn check() {
        let mut hasher = super::SHA3_224.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("b87f88c72702fff1748e58b87e9141a42c0dbedc29a78cb0d4a5cd81")
        );

        let mut hasher = super::SHA3_256.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392")
        );

        let mut hasher = super::SHA3_384.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(hasher.finalize(), hex!("720aea11019ef06440fbf05d87aa24680a2153df3907b23631e7177ce620fa1330ff07c0fddee54699a4c3ee0ee9d887"));

        let mut hasher = super::SHA3_512.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(hasher.finalize(), hex!("75d527c368f2efe848ecf6b073a36767800805e9eef2b1857d5f984f036eb6df891d75f72d9b154518c1cd58835286d1da9a38deba3de98b5a53e5ed78a84976"));
    }
}
