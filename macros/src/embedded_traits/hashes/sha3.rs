use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use syn::parse_quote;

use super::{digest::DigestHashAlg, HashTrait};

pub fn sha3_224_trait() -> HashTrait<DigestHashAlg<Sha3_224>> {
    HashTrait::new(DigestHashAlg::new(
        "sha3-224",
        parse_quote!(::embed_it::Sha3_224Hash),
        parse_quote!(sha3_224),
    ))
}

pub fn sha3_256_trait() -> HashTrait<DigestHashAlg<Sha3_256>> {
    HashTrait::new(DigestHashAlg::new(
        "sha3-256",
        parse_quote!(::embed_it::Sha3_256Hash),
        parse_quote!(sha3_256),
    ))
}

pub fn sha3_384_trait() -> HashTrait<DigestHashAlg<Sha3_384>> {
    HashTrait::new(DigestHashAlg::new(
        "sha3-384",
        parse_quote!(::embed_it::Sha3_384Hash),
        parse_quote!(sha3_384),
    ))
}

pub fn sha3_512_trait() -> HashTrait<DigestHashAlg<Sha3_512>> {
    HashTrait::new(DigestHashAlg::new(
        "sha3-512",
        parse_quote!(::embed_it::Sha3_512Hash),
        parse_quote!(sha3_512),
    ))
}

#[cfg(test)]
mod tests {

    use super::*;
    use hex_literal::hex;

    #[test]
    fn check() {
        use super::super::{HashAlg, Hasher};
        use hex_literal::hex;
        let mut hasher = sha3_224_trait().unwrap().alg.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("b87f88c72702fff1748e58b87e9141a42c0dbedc29a78cb0d4a5cd81")
        );

        let mut hasher = sha3_256_trait().unwrap().alg.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392")
        );

        let mut hasher = sha3_384_trait().unwrap().alg.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(hasher.finalize(), hex!("720aea11019ef06440fbf05d87aa24680a2153df3907b23631e7177ce620fa1330ff07c0fddee54699a4c3ee0ee9d887"));

        let mut hasher = sha3_512_trait().unwrap().alg.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(hasher.finalize(), hex!("75d527c368f2efe848ecf6b073a36767800805e9eef2b1857d5f984f036eb6df891d75f72d9b154518c1cd58835286d1da9a38deba3de98b5a53e5ed78a84976"));
    }
}
