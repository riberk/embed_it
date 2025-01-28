use md5::Md5;
use syn::parse_quote;

use super::{digest::DigestHashAlg, HashTrait};

pub fn md5_trait() -> HashTrait<DigestHashAlg<Md5>> {

    HashTrait::new(DigestHashAlg::new(
        "md5",
        parse_quote!(::embed_it::Md5Hash),
        parse_quote!(md5),
    ))
}

#[cfg(test)]
mod tests {

    use super::md5_trait;

    #[test]
    #[cfg(feature = "md5")]
    fn check_md5() {
        use super::super::{HashAlg, Hasher};
        use hex_literal::hex;
        let mut hasher = md5_trait().alg.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(hasher.finalize(), hex!("5d41402abc4b2a76b9719d911017c592"));
    }
}
