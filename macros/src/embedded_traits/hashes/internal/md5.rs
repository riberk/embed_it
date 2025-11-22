use md5::Md5;
use syn::parse_quote;

use crate::embedded_traits::hashes::ids;

use super::{HashTrait, digest::DigestHashAlg};

pub const MD5: &HashTrait<DigestHashAlg<Md5>> = &HashTrait::new(DigestHashAlg::new(
    ids::MD5.id,
    || parse_quote!(::embed_it::Md5Hash),
    || parse_quote!(md5),
));

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::super::{HashAlg, Hasher};
    use super::MD5;

    #[test]
    fn check() {
        let mut hasher = MD5.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(hasher.finalize(), hex!("5d41402abc4b2a76b9719d911017c592"));
    }
}
