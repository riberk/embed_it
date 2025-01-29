#[derive(embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/templates",
    file(derive_default_traits = false, derive(StrContent),),
    dir(derive_default_traits = false)
)]
pub struct Templates;

#[derive(serde::Serialize)]
pub struct DirModel<'a> {
    pub title: &'a str,
    pub entries: Vec<EntryModel<'a>>,
}

#[derive(serde::Serialize)]
pub struct EntryModel<'a> {
    pub href: &'a str,
    pub title: &'a str,
    pub icon_src: &'a str,
}
