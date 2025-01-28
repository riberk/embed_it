use blake3::Hasher;
use syn::parse_quote;

use super::{HashAlg, HashTrait};

#[derive(Debug)]
pub struct Blake3HashAlg {
    len: usize,
    id: String,
}

impl Blake3HashAlg {
    pub fn new(len: usize) -> Self {
        Self {
            len,
            id: format!("blake3-{len}"),
        }
    }
}

struct Blake3Hasher {
    hasher: Hasher,
    len: usize,
}

impl std::io::Write for Blake3Hasher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.hasher.flush()
    }
}

impl super::Hasher for Blake3Hasher {
    fn hash(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len);
        self.0.finalize_xof().fill(&mut buf);
        buf
    }
}

impl HashAlg for Blake3HashAlg {
    fn id(&self) -> &str {
        &self.id
    }

    fn trait_path(&self) -> syn::Path {
        parse_quote!(::embed_it::Blake3_256Hash)
    }

    fn trait_method(&self) -> syn::Ident {
        parse_quote!(blake3_256)
    }

    fn hash_len(&self) -> usize {
        self.len
    }

    fn make_hasher(&self) -> impl super::Hasher {
        Blake3Hasher {
            hasher: Hasher::new(),
            len: self.len,
        }
    }
}

pub const BLAKE3: &HashTrait<Blake3HashAlg> = &HashTrait::new(Blake3HashAlg);

pub fn blake3_trait(len: usize) -> HashTrait<Blake3HashAlg> {
    
}

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
