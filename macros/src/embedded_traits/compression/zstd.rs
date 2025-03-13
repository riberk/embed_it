use syn::parse_quote;

use super::{CompressionAlg, CompressionTrait, Compressor, FinalizeCompressorError, ids};

#[derive(Debug)]
pub struct Zstd;

impl Compressor for zstd::stream::Encoder<'_, Vec<u8>> {
    fn finalize(self) -> Result<Vec<u8>, FinalizeCompressorError> {
        self.finish().map_err(FinalizeCompressorError::Io)
    }
}

impl CompressionAlg for Zstd {
    fn id(&self) -> &'static str {
        ids::ZSTD.id
    }

    fn trait_path(&self) -> syn::Path {
        parse_quote!(::embed_it::ZstdContent)
    }

    fn trait_method(&self) -> syn::Ident {
        parse_quote!(zstd_content)
    }

    fn make_compressor(&self) -> impl Compressor {
        zstd::stream::Encoder::new(Vec::new(), 22).unwrap()
    }
}
pub const ZSTD: &CompressionTrait<Zstd> = &CompressionTrait::new(Zstd);

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use hex_literal::hex;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn check() {
        let mut compressor = Zstd.make_compressor();
        let source = b"00000000000000000000000000000000";
        compressor.write_all(source).unwrap();
        let result = compressor.finalize().unwrap();

        assert_eq!(&result, &hex!("28b52ffd0088450000103030010022c002"));
        let mut decoder = zstd::stream::Decoder::new(result.as_slice()).unwrap();
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();

        assert_eq!(&source, &decoded.as_slice());
    }
}
