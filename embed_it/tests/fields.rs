use std::str::from_utf8;

#[derive(embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    dir(field(name = "children", factory = Children, global)),
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
        data.entries()
            .iter()
            .map(|v| v.as_ref().map(|d| d.path(), |f| f.path()).value().name())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use embed_it::Index;

    use crate::{AsStrField, Assets, ChildrenField};

    #[test]
    fn additional_fields() {
        assert_eq!(Assets.one_txt().children(), &vec!["hello", "world"]);

        assert_eq!(Assets.one_txt().hello().as_str(), &Some("hello"));

        assert_eq!(
            Assets.get("one_txt").unwrap().dir().unwrap().children(),
            &vec!["hello", "world"]
        );
    }
}
