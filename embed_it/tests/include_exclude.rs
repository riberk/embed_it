#[derive(embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    dir(
        derive_default_traits = false,
        exclude(pattern = "*_txt"),
        derive(Path),
        derive(Index),
    ),
    file(derive_default_traits = false, include(regex = ".*e.*"), derive(Path),)
)]
pub struct Assets;

#[cfg(test)]
mod tests {

    use embed_it::Index;

    use crate::Assets;

    #[test]
    fn include_exclude() {
        assert!(Assets.get("one.txt").is_some());
        assert!(Assets.get("hello.txt").is_some());

        assert!(Assets.get("world.txt").is_none());
        assert!(Assets.get("one_txt").is_none());
    }
}
