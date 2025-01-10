use macros::EmbeddedDir;

#[derive(EmbeddedDir)]
#[embedded_dir(path = "$CARGO_MANIFEST_DIR/assets")]
pub struct Assets;
