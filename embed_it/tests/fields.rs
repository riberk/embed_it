use std::str::from_utf8;

use embed_it::EntryPath;

#[derive(embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    dir(field(name = "children", factory = Children)),
    file(field(name = "as_str", factory = AsStr))
)]
pub struct Assets;

pub struct AsStr;

impl FileFieldFactory for AsStr {
    type Field = Option<&'static str>;

    fn create<T: File + ?Sized>(data: &T) -> Self::Field {
        from_utf8(data.content()).ok()
    }
}

pub struct Children;

impl DirFieldFactory for Children {
    type Field = Vec<&'static str>;

    fn create<T: Dir + ?Sized>(data: &T) -> Self::Field {
        data.entries().iter().map(|v| v.path().name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{AsStrField, Assets, ChildrenField};

    #[test]
    fn additional_fields() {
        assert_eq!(Assets.one_txt().children(), &vec!["hello", "world"]);

        assert_eq!(Assets.one_txt().hello().as_str(), &Some("hello"));
    }
}
