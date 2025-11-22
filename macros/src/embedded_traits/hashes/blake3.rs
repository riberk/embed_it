use blake3::Hasher;
use syn::parse_quote;

use super::{HashAlg, HashTrait};

#[derive(Debug)]
pub struct Blake3HashAlg;

struct Blake3Hasher(Hasher);

impl std::io::Write for Blake3Hasher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl super::Hasher for Blake3Hasher {
    fn hash(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        let res: [u8; blake3::OUT_LEN] = self.0.finalize().into();
        res.into()
    }
}

impl HashAlg for Blake3HashAlg {
    fn id(&self) -> &'static str {
        super::ids::BLAKE3.id
    }

    fn trait_path(&self) -> syn::Path {
        parse_quote!(::embed_it::Blake3_256Hash)
    }

    fn trait_method(&self) -> syn::Ident {
        parse_quote!(blake3_256)
    }

    fn make_hasher(&self) -> impl super::Hasher {
        Blake3Hasher(Hasher::new())
    }

    fn output_size(&self) -> usize {
        blake3::OUT_LEN
    }
}

pub const BLAKE3: &HashTrait<Blake3HashAlg> = &HashTrait::new(Blake3HashAlg);

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::super::{HashAlg, Hasher};

    #[test]
    fn check() {
        let mut hasher = super::BLAKE3.0.make_hasher();
        hasher.hash(b"hello");
        assert_eq!(
            hasher.finalize(),
            hex!("ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f")
        );
    }
}
