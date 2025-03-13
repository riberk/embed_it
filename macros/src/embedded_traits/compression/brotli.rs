use syn::parse_quote;

use super::{CompressionAlg, CompressionTrait, Compressor, FinalizeCompressorError, ids};

#[derive(Debug)]
pub struct Brotli;

impl Compressor for brotli::CompressorWriter<Vec<u8>> {
    fn finalize(self) -> Result<Vec<u8>, FinalizeCompressorError> {
        Ok(self.into_inner())
    }
}

impl CompressionAlg for Brotli {
    fn id(&self) -> &'static str {
        ids::BROTLI.id
    }

    fn trait_path(&self) -> syn::Path {
        parse_quote!(::embed_it::BrotliContent)
    }

    fn trait_method(&self) -> syn::Ident {
        parse_quote!(brotli_content)
    }

    fn make_compressor(&self) -> impl Compressor {
        brotli::CompressorWriter::new(Vec::new(), 1024 * 1024, 11, 22)
    }
}

pub const BROTLI: &CompressionTrait<Brotli> = &CompressionTrait::new(Brotli);

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use hex_literal::hex;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn check() {
        let mut compressor = Brotli.make_compressor();
        let source = b"00000000000000000000000000000000";
        compressor.write_all(source).unwrap();
        let result = compressor.finalize().unwrap();

        assert_eq!(&result, &hex!("1b1f00f82560828c00c0"));
        let mut decompressor = brotli::Decompressor::new(result.as_slice(), 0);
        let mut decoded = Vec::new();
        decompressor.read_to_end(&mut decoded).unwrap();

        assert_eq!(&source, &decoded.as_slice());
    }
}
