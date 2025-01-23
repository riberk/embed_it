use ::embed_it::ChildOf;
#[derive(::embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    dir(mark(ChildOf)),
    file(mark(ChildOf))
)]
pub struct Assets;

fn child_content<T: ChildOf<Assets, 0> + File>(value: &T) -> &'static [u8] {
    value.content()
}

fn grandchild_content<T: ChildOf<Assets, 1> + File>(value: &T) -> &'static [u8] {
    value.content()
}

#[test]
fn check() {
    assert_eq!(grandchild_content(Assets.one_txt().hello()), b"hello");
    assert_eq!(grandchild_content(Assets.one_txt().world()), b"world");
    assert_eq!(child_content(Assets.one()), b"one");

    // it is not compiled, because `Assets.one_txt().world()`
    // implements `ChildOf<Assets, 1>` and `ChildOf<OneTxt, 0>`, but not `ChildOf<Assets, 0>`
}
