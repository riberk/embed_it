use sha1::Sha1;
use syn::parse_quote;

use super::{digest::DigestHashAlg, HashTrait};

pub const SHA1: &HashTrait<DigestHashAlg<Sha1>> = &HashTrait::new(DigestHashAlg::new(
    super::ids::SHA1.id,
    || parse_quote!(::embed_it::Sha1Hash),
    || parse_quote!(sha1),
));

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::super::{HashAlg, Hasher};

    #[test]
    fn check() {
        let mut hasher = super::SHA1.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d")
        );
    }
}
