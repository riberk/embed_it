use flate2::{Compression, write::GzEncoder};
use syn::parse_quote;

use super::{CompressionAlg, CompressionTrait, Compressor, FinalizeCompressorError, ids};

#[derive(Debug)]
pub struct Gzip;

impl Compressor for GzEncoder<Vec<u8>> {
    fn finalize(self) -> Result<Vec<u8>, FinalizeCompressorError> {
        self.finish().map_err(FinalizeCompressorError::Io)
    }
}

impl CompressionAlg for Gzip {
    fn id(&self) -> &'static str {
        ids::GZIP.id
    }

    fn trait_path(&self) -> syn::Path {
        parse_quote!(::embed_it::GzipContent)
    }

    fn trait_method(&self) -> syn::Ident {
        parse_quote!(gzip_content)
    }

    fn make_compressor(&self) -> impl super::Compressor {
        GzEncoder::new(Vec::new(), Compression::new(9))
    }
}

pub const GZIP: &CompressionTrait<Gzip> = &CompressionTrait::new(Gzip);

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use flate2::read::GzDecoder;
    use hex_literal::hex;
    use pretty_assertions::assert_eq;

    use crate::embedded_traits::compression::{CompressionAlg, Compressor, gzip::Gzip};

    #[test]
    fn check() {
        let mut compressor = Gzip.make_compressor();
        let source = b"00000000000000000000000000000000";
        compressor.write_all(source).unwrap();
        let result = compressor.finalize().unwrap();

        assert_eq!(
            &result,
            &hex!("1f8b08000000000002ff3330c00f00d092f2a020000000")
        );
        let mut gz = GzDecoder::new(result.as_slice());
        let mut decoded = Vec::new();
        gz.read_to_end(&mut decoded).unwrap();

        assert_eq!(&source, &decoded.as_slice());
    }
}
