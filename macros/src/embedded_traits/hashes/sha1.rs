use sha1::Sha1;
use syn::parse_quote;

use super::{digest::DigestHashAlg, HashTrait};

pub fn sha1_trait() -> HashTrait<DigestHashAlg<Sha1>> {
    HashTrait::new(DigestHashAlg::new(
        "sha1",
        parse_quote!(::embed_it::Sha1Hash),
        parse_quote!(sha1),
    ))
}

#[cfg(test)]
mod tests {

    use super::sha1_trait;

    #[test]
    fn check_md5() {
        use super::super::{HashAlg, Hasher};
        use hex_literal::hex;
        let mut hasher = sha1_trait().unwrap().alg.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d")
        );
    }
}
