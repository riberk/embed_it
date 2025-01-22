# embed_it
[![Build Status](https://github.com/riberk/embed_it/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/riberk/embed_it/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/embed_it.svg)](https://crates.io/crates/embed_it)
[![Coverage](https://riberk.github.io/embed_it/badges/coverage.svg)](https://riberk.github.io/embed_it/coverage_report/index.html)

Include any directory as a struct, and the entire tree will be generated as Rust structures and traits

Imagine a project structure like this:

- **assets/**
  - **one_txt/**
    - hello
    - world
  - hello.txt
  - one.txt
  - world.txt
- src
- Cargo.toml

You can use a macro to expand it into Rust code:

```rust
use embed_it::Embed;
#[derive(Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    support_alt_separator,
)]
pub struct Assets;

# fn main() {
    use embed_it::{Content, EntryPath, EmbeddedPath, Index};
    assert_eq!(Assets.hello().content(), b"hello");
    assert_eq!(Assets.hello().path(), &EmbeddedPath::new("hello.txt", "hello.txt", "hello"));

    assert_eq!(Assets.one().content(), b"one");
    assert_eq!(Assets.one().path(), &EmbeddedPath::new("one.txt", "one.txt", "one"));

    assert_eq!(Assets.world().content(), b"world");
    assert_eq!(Assets.world().path(), &EmbeddedPath::new("world.txt", "world.txt", "world"));

    assert_eq!(Assets.one_txt().path(), &EmbeddedPath::new("one_txt", "one_txt", "one_txt"));

    assert_eq!(Assets.one_txt().hello().content(), b"hello");
    assert_eq!(Assets.one_txt().hello().path(), &EmbeddedPath::new("one_txt/hello", "hello", "hello"));

    assert_eq!(Assets.one_txt().world().content(), b"world");
    assert_eq!(Assets.one_txt().world().path(), &EmbeddedPath::new("one_txt/world", "world", "world"));

    // or with dynamic dispatch
    assert_eq!(
        Assets.get("one_txt/hello").unwrap().file().unwrap().content(),
        b"hello"
    );

    // We can use Windows-style paths due to the `support_alt_separator` attribute
    assert_eq!(
        Assets.get("one_txt\\hello").unwrap().file().unwrap().content(),
        b"hello"
    );
# }
```

## Known issues

### Long compilation time with many files
If your directory contains a very large number of files, the compile time can increase significantly.

**Possible solution**: Move those assets into a separate crate. This way, the main build won’t be slowed down by the large amount of embedded content, and changes in the asset crate won’t force a full rebuild of your main project.

### `macro invocation exceeds token limit` error in rust-analyzer
When there are thousands of files/directories (around 5000 or more), rust-analyzer can fail with the error that the macro exceeds the token limit. This is due to a hard-coded limit in rust-analyzer that is not currently configurable [tracking issue](https://github.com/rust-lang/rust-analyzer/issues/10855).

**Possible workaround**: Split the assets into multiple directories and generate several smaller embedded structures, each containing fewer files, to reduce the total token count.

### Intellisense issues in RustRover
In JetBrains RustRover, intellisense might stop working when the number of files/directories reaches a similar high threshold. The exact cause and any permanent solution are currently unclear.

**Possible workaround**: As above, splitting assets into multiple directories with separate macro invocations may help avoid hitting internal limits.

## Fields

### embed

The main attribute

| field                    | type             | multiple | required | default                | description                                                                                                                                                                                                                                                                                 |
|--------------------------|------------------|----------|----------|------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `path`                   | `String`         | false    | true     | -                      | The path to the directory with assets. It may contain [compile-time environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates) (or user defined) in format `$CARGO_MANIFEST_DIR` or `${CARGO_MANIFEST_DIR}`   |
| `dir`                    | `DirAttr`        | false    | false    | `DirAttr::default()`   | Changes the setting for how the `Dir`trait and its implementations are generated. See more in the [Dir Attr](#DirAttr) section                                                                                                                                                              |
| `file`                   | `FileAttr`       | false    | false    | `FileAttr::default()`  | Changes the setting for how the `File` trait and its implementations are generated. See more in the [File Attr](#FileAttr) section                                                                                                                                                          |
| `entry`                  | `EntryAttr`      | false    | false    | `EntryAttr::default()` | Changes the setting for how the `Entry` struct and its implementations are generated. See more in the [Entry Attr](#EntryAttr) section                                                                                                                                                      |
| `with_extension`         | `bool`           | false    | false    | `false`                | Use file extensions for method and struct names                                                                                                                                                                                                                                             |
| `support_alt_separator`  | `bool`           | false    | false    | `false`                | If true, getting a value from the directory's `Index` replaces `\` with `/`. In other words, you can use Windows-style paths with the `get` method, for example, `Assets.get("a\\b\\c.txt")`                                                                                                |


### <a name="DirAttr"></a> DirAttr

| field                      | type             | multiple | required | default                                      | description                                                                                                                                                                      |
|----------------------------|------------------|----------|----------|----------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `derive_default_traits`    | `bool`           | false    | false    | `true`                                       | Determines whether default traits will be derived (see the `derive` row in the table)                                                                                                                   |
| `trait_name`               | `Ident`          | false    | false    | `Dir`                                        | Specifies the trait name that will be used for a directory                                                                                                                                     |
| `field_factory_trait_name` | `Ident`          | false    | false    | `DirFieldFactory`                            | Specifies the trait name that will be used for a directory field factory                                                                                                                       |
| `derive`                   | `Vec<DirTrait>`  | true     | false    | `Path`, `Entries`, `Index`, `Meta`, `Debug` </br> `DirectChildCount`, `RecursiveChildCount` | What traits will be derived for every directory and what bounds will be set for the `Dir` trait. See also [EmbeddedTraits list](#EmbeddedTraits_list) and [Hash traits](#HashTraits) |
| `field`                    | `Vec<FieldAttr>` | true     | false    | `vec![]`                                     | Adds additional fields for a directory. See more in the [Field Attr](#FieldAttr) section                                                                                                            |

### <a name="FileAttr"></a> FileAttr

| field                      | type             | multiple | required | default                            | description                                                                                                                                                                                                             |
|----------------------------|------------------|----------|----------|------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `derive_default_traits`    | `bool`           | false    | false    | `true`                             | Determines whether default traits will be derived (see the `derive` row in the table)                                                                                                                                   |
| `trait_name`               | `Ident`          | false    | false    | `File`                             | What trait name will be used for a directory                                                                                                                                                                            |
| `field_factory_trait_name` | `Ident`          | false    | false    | `FileFieldFactory`                 | What trait name will be used for a directory field factory                                                                                                                                                              |
| `derive`                   | `Vec<DirTrait>`  | true     | false    | `Path`, `Meta`, `Debug`, `Content` | What traits will be derived for every directory and what bounds will be set for a Dir trait. See also [EmbeddedTraits list](#EmbeddedTraits_list), [Hash traits](#HashTraits), [Compression traits](#CompressionTraits) |
| `field`                    | `Vec<FieldAttr>` | true     | false    | `vec![]`                           | Adds additional fields for a file. See more in the [Field Attr](#FieldAttr) section                                                                                                                                     |

### <a name="EmbeddedTraits_list"></a> EmbeddedTraits list

| **name**                | **trait**                      | **dir or file** | **method**                                            | **purpose**                                                                                                                                                       |
|-------------------------|--------------------------------|-----------------|-------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Path**                | [`crate::EntryPath`]           | any             | `fn path(&self) -> &'static EmbeddedPath;`            | Provides full information about a path of an entry                                                                                                                |
| **Entries**             | *\<auto generated\>*           | dir             | `fn entries(&self) -> &'static [Entry]`               | Provides direct children of a dir                                                                                                                                 |
| **Index**               | *\<auto generated\>*           | dir             | `fn get(&self, path: &str) -> Option<&'static Entry>` | Provides fast access (`HashMap`) to all children (recursively). It constructs hash set on every level dir and might use some memory if there are a lot of entries |
| **DirectChildCount**    | [`crate::DirectChildCount`]    | dir             | `fn direct_child_count(&self) -> usize;`              | Provides the number of direct children                                                                                                                            |
| **RecursiveChildCount** | [`crate::RecursiveChildCount`] | dir             | `fn recursive_child_count(&self) -> usize;`           | Provides the total number of children, including nested subdirectories                                                                                            |
| **Meta**                | [`crate::Meta`]                | any             | `fn metadata(&self) -> &'static Metadata;`            | Provides metadata of an entry                                                                                                                                     |
| **Debug**               | [`std::fmt::Debug`]            | any             |                                                       | Debugs structs                                                                                                                                                    |
| **Content**             | [`crate::Content`]             | file            | `fn content(&self) -> &'static [u8];`                 | Provides content of a file                                                                                                                                        |
| **Hashes**              | *\<various\>*                  | any             | `fn <name>[<_bits>](&self) -> &'static [u8; <bits>];` | Provides hash of a file content or a directory structure with files' hashes. See also [Hash traits](#HashTraits)                                                  |
| **Compression**         | *\<various\>*                  | file            | `fn <name>_content(&self) -> &'static [u8];`          | Provides the compressed content of a file. See also [Compression traits](#CompressionTraits)                                                                      |

### <a name="EntryAttr"></a> EntryAttr
| field                      | type            | multiple | required | default                                     | description                                                                                                                                      |
|----------------------------|-----------------|----------|----------|---------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| `struct_name`              | `Ident`         | false    | false    | `Entry`                                     | What struct name will be used for an entry                                                                                                       |

### <a name="FieldAttr"></a> FieldAttr

You can add any additional fields, which will be created in runtime (but only once) from a dir or a file. 
For each `field` defined in macros a special trait will be generated inside the module containing a root structure.

| field                      | type            | multiple | required | default                                     | description                                                                                                                                      |
|----------------------------|-----------------|----------|----------|---------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| `name`                     | `Ident`         | false    | true     |                                             | The name of the method that will be used by the trait                                                                                            |
| `factory`                  | `syn::Path`     | false    | true     |                                             | The path to a factory, that will be used to create an instance of the field and to determine a field type                                        |
| `trait_name`               | `Option<Ident>` | false    | false    | `{name.to_pascal_case()}Field`              | The name of the field trait                                                                                                                      |
| `regex`                    | `Option<String>`| false    | false    | `None`                                      | Regular expression to match a fs entry path. The trait is implemented for a struct only if the regex matches                                |
| `pattern`                  | `Option<String>`| false    | false    | `None`                                      | Glob pattern to match a fs entry path. The trait is implemented for a struct only if the pattern matches                                    |

```rust
use std::str::from_utf8;
use embed_it::Embed;

#[derive(Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    file(
        field(
            // it is a trait method name used to get an instance of a field.
            // you can use your own name for the trait with attribute `trait_name`.
            // By default it is `{name.to_pascal()}Field`.
            // In that case it will be `AsStrField`.
            name = "as_str", 
            
            // factory is a path to the struct implementing either
            // a trait self::FileFieldFactory for target = "file"
            // or a trait self::DirFieldFactory for target = "dir"
            factory = AsStr,   
            
            // glob pattern
            pattern = "*.txt",
        ), 
    ),
    dir(
        field(
            name = "children", 
            factory = crate::Children, 
            regex = ".+_txt",
        ), 
        field(
            name = "root_children", 
            trait_name = "Root",
            factory = crate::Children, 
            // this trait will be implemented only for root struct (`Assets`)
            regex = ""
        ), 
    ),

)]
pub struct Assets;


pub struct AsStr(&'static str);
impl FileFieldFactory for AsStr {
    type Field = Option<Self>;

    fn create<T: File + ?Sized>(data: &T) -> Self::Field {
        use embed_it::{ Content };
        from_utf8(data.content()).map(AsStr).ok()
    }
}

pub struct Children;
impl DirFieldFactory for Children {
    type Field = Vec<&'static str>;

    fn create<T: Dir + ?Sized>(data: &T) -> Self::Field {
        use embed_it::{ EntryPath, Entries };
        data.entries().iter().map(|e| e.map(|d| d.path(), |f| f.path()).value().name()).collect()
    }
}

# fn main() {
use embed_it::{ Content };

// the first field `as_str`
use AsStrField;
assert_eq!(Assets.hello().content(), b"hello");
assert_eq!(Assets.one().content(), b"one");
assert_eq!(Assets.world().content(), b"world");

assert_eq!(Assets.hello().as_str().as_ref().unwrap().0, "hello");
assert_eq!(Assets.one().as_str().as_ref().unwrap().0, "one");
assert_eq!(Assets.world().as_str().as_ref().unwrap().0, "world");

// this is not compile due to `pattern` (`one_txt/hello` has no extension)
// Assets.one_txt().as_str()

// the second field `children`
use ChildrenField;
assert_eq!(Assets.one_txt().children(), &vec!["hello", "world"]);

// the third field `root_children`
use Root;
assert_eq!(Assets.root_children(), &vec!["one_txt", "hello.txt", "one.txt", "world.txt"]);
# }

```

### <a name="HashTraits"></a> Hash traits

You can use any combination of hash traits on `dir` and `file`. For a file, it hashes its content; for a directory, it hashes every entry name and entry hash if applicable (order — directories first, then files, and finally by path). The hash is stored as a constant array of bytes.

| Derive     | Required feature | Trait                     | 
|------------|------------------|---------------------------| 
| `Md5`      | `md5`            | [`crate::Md5Hash`]        | 
| `Sha1`     | `sha1`           | [`crate::Sha1Hash`]       | 
| `Sha2_224` | `sha2`           | [`crate::Sha2_224Hash`]   | 
| `Sha2_256` | `sha2`           | [`crate::Sha2_256Hash`]   | 
| `Sha2_384` | `sha2`           | [`crate::Sha2_384Hash`]   | 
| `Sha2_512` | `sha2`           | [`crate::Sha2_512Hash`]   | 
| `Sha3_224` | `sha3`           | [`crate::Sha3_224Hash`]   | 
| `Sha3_256` | `sha3`           | [`crate::Sha3_256Hash`]   | 
| `Sha3_384` | `sha3`           | [`crate::Sha3_384Hash`]   | 
| `Sha3_512` | `sha3`           | [`crate::Sha3_512Hash`]   | 
| `Blake3`   | `blake3`         | [`crate::Blake3_256Hash`] | 

The example below compiles only if all hash features listed in the table above are enabled.

```rust
#[cfg(
    all(
        feature = "md5",
        feature = "sha1",
        feature = "sha2",
        feature = "sha3",
        feature = "blake3"
    )
)]
mod lib {
    use std::str::from_utf8;
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        dir(
            derive(Md5),
            derive(Sha1),
            derive(Sha2_224),
            derive(Sha2_256),
            derive(Sha2_384),
            derive(Sha2_512),
            derive(Sha3_224),
            derive(Sha3_256),
            derive(Sha3_384),
            derive(Sha3_512),
            derive(Blake3),
        ),
        file(
            derive(Md5),
            derive(Sha1),
            derive(Sha2_224),
            derive(Sha2_256),
            derive(Sha2_384),
            derive(Sha2_512),
            derive(Sha3_224),
            derive(Sha3_256),
            derive(Sha3_384),
            derive(Sha3_512),
            derive(Blake3),
        ),
    )]
    pub struct Assets;

    # fn main() {
    use embed_it::{ 
        Md5Hash, 
        Sha1Hash, 
        Sha2_224Hash, 
        Sha2_256Hash, 
        Sha2_384Hash, 
        Sha2_512Hash,
        Sha3_224Hash, 
        Sha3_256Hash, 
        Sha3_384Hash, 
        Sha3_512Hash,
        Blake3_256Hash,
        Entries
    };

    use hex_literal::hex;

    assert_eq!(Assets.md5(), &hex!("56e71a41c76b1544c52477adf4c8e2f7"));
    assert_eq!(Assets.sha1(), &hex!("26da80338f55108be5bcce49285a4154f6705599"));
    assert_eq!(Assets.sha2_224(), &hex!("360c16e2d8135a337cc6ddf4134ec9cc69dd65b779db2a2807f941e4"));
    assert_eq!(Assets.sha2_256(), &hex!("e16b758a01129c86f871818a7b4e31c88a3c6b69d9c8319bcbc881b58f067b25"));
    assert_eq!(Assets.sha2_384(), &hex!("de4656a27347eee72aea1d15e85f20439673709cde5339772660bbd9d800bbde9f637eb3505f572140432625f3948175"));
    assert_eq!(Assets.sha2_512(), &hex!("bc1673b560316c6586fa1ec98ca5df3e303b66ddae944b05c71314806f88bd4b8f4c7832dfb7dd729eaca191b7142936d21bd07f750c9bc35d67f218e51bbaa4"));
    assert_eq!(Assets.sha3_224(), &hex!("6949265b40fa55e0c194e3591f90e6cbf0ac100d7ed32e71d6e1e753"));
    assert_eq!(Assets.sha3_256(), &hex!("a2d99103dc2d1967fb05c4de99a1432e9afb1f5acc698fefb2112ce7fb9335c4"));
    assert_eq!(Assets.sha3_384(), &hex!("cf1f50cb53dc61b3519227887bfb20230b6878d32b10c5a9bfe016095aaecc593e612a165c89488109da62138a7214d8"));
    assert_eq!(Assets.sha3_512(), &hex!("aeff4601a53fecdad418f3245676398719d507bd7b971098ad3f4c2d495c2cc96faf022f481c0bebc0632492abd8eb9fe9f8af6d25664f33d61ff316d269682a"));
    assert_eq!(Assets.blake3_256(), &hex!("b5947e2140b0fe744b1afe9a9f9031e72571c85db079413a67b4a9309f581de7"));

    # }
}


```

### <a name="CompressionTraits"></a> Compression traits

You can use any combination of compression traits on a `file`. It stores compressed content with provided algorythm.

It might help you to use in a case like providing static content from a web server - you can analyze `Accept` header and use it to provide various `Content-Encoding` and body. See it in [examples](./examples/web)

| Derive     | Required feature | Trait                     | Compression settings                          | 
|------------|------------------|---------------------------|-----------------------------------------------| 
| `Zstd`     | `zstd`           | [`crate::ZstdContent`]    | Compression level = 19                        | 
| `Gzip`     | `gzip`           | [`crate::GzipContent`]    | Compression level = 9                         | 
| `Brotli`   | `brotli`         | [`crate::BrotliContent`]  | Compression level = 11, LZ77 window size = 22 | 


```rust
#[cfg(
    all(
        feature = "zstd",
        feature = "gzip",
        feature = "brotli",
    )
)]
mod lib {
    use std::str::from_utf8;
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        file(
            derive(Zstd),
            derive(Gzip),
            derive(Brotli),
        ),
    )]
    pub struct Assets;

    # fn main() {
    use embed_it::{ 
        BrotliContent, 
        GzipContent, 
        ZstdContent, 
    };

    use hex_literal::hex;

    assert_eq!(Assets.hello().gzip_content(), &hex!("1f8b08000000000002ffcb48cdc9c9070086a6103605000000"));
    assert_eq!(Assets.hello().zstd_content(), &hex!("28b52ffd008829000068656c6c6f"));
    assert_eq!(Assets.hello().brotli_content(), &hex!("0b028068656c6c6f03"));

    # }
}


```

## More complex example

```rust
use embed_it::Embed;
#[derive(Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    dir(
        // trait name for directories (default `Dir`)
        trait_name = AssetsDir, 
        
        // trait name for directory field's factories (default `DirFieldFactory`)
        field_factory_trait_name = AssetsDirFieldFactory, 
        
        // Do not derive default traits for a dir
        derive_default_traits = false,

        // implement embed_it::EntryPath
        derive(Path), 
        
        // implement `Entries` trait, which stores all direct children into an array
        derive(Entries), 
        
        // implement `Index` trait, which stores (recursively) all children into a set
        derive(Index), 

        // implement `embed_it::Meta` trait, which provides metadata of the entry
        derive(Meta),
        
        // implement `std::fmt::Debug` for directory. It writes each child implementing debug
        derive(Debug),
        field(
            name = children, 
            trait_name = AssetsChildrenField, 
            factory = Children, 
            pattern = "?*", 
            regex = ".+", 
        ),
        field(
            name = root_children, 
            trait_name = AssetsRootChildrenField, 
            factory = Children, 
            // only for `Assets`
            regex = "", 
        ),
    ),
    file(
        // trait name for files (default `File`)
        trait_name = AssetsFile, 
        
        // trait name for file field's factories (default `FileFieldFactory`)
        field_factory_trait_name = AssetsFileFieldFactory, 

        // Do not derive default traits for a file
        derive_default_traits = false,

        // implement embed_it::EntryPath
        derive(Path), 
        
        // implement `embed_it::Meta` trait, which provides metadata of the entry
        derive(Meta),
        
        // implement `embed_it::Content` trait, which provides content of the file as a byte array
        derive(Content),
        
        // implement `std::fmt::Debug` for a file. It writes Content len
        derive(Debug),
        field(
            // The name of the method of the trait
            name = as_str, 
            
            // The trait name, defaul `"{name.to_pascal()}Field"`
            trait_name = AssetsAsStrField, 
            
            // The factory to create an instance of the field
            factory = AsStr, 
            
            // The pattern to match entry's path. Default None
            pattern = "*.txt", 
            
            // The regex to match entry's path. Default None
            regex = ".+", 
        ),
    ),
    // `Entry` - enum with `Dir(&'static dyn Dir)/File(&'static dyn File)` variants
    // `Entry` implements intersection of `Dir`'s and `File`'s traits
    entry(
        // struct name for a param of the `Entry::Dir()`. Default `DynDir`
        dir_struct_name = DynDir,
        
        // struct name for a param of the `Entry::File()`. Default `DynDir`
        file_struct_name = DynFile,
        
        // trait name for a trait which is combination of the `Dir` and all `global` fields. Default `EntryDir`
        dir_trait_name = EntryDir,
        
        // trait name for a trait which is combination of the `File` and all `global` fields. Default `EntryFile`
        file_trait_name = EntryFile,
    ),
    // if true, the macro will use the extension as a part of `StructName`s and `method_name`s
    // e.g. hello.txt turns into HelloTxt/hello_txt() if with_extension = true, and Hello/hello() if with_extension = false
    // default is false
    with_extension = true,

    
)]
pub struct Assets;

pub struct Children;

// The name of the factory as in the attribute `dir`
impl AssetsDirFieldFactory for Children {
    type Field = Vec<&'static str>;

    fn create<T: AssetsDir + ?Sized>(data: &T) -> Self::Field {
        use embed_it::EntryPath;
        data.entries().iter().map(|v| v.map(|d| d.path(), |f| f.path()).value().relative_path_str()).collect()
    }
}

pub struct AsStr;

// The name of the factory as in the attribute `file`
impl AssetsFileFieldFactory for AsStr {
    type Field = Option<&'static str>;

    fn create<T:AssetsFile + ?Sized>(data: &T) -> Self::Field {
        std::str::from_utf8(data.content()).ok()
    }
}

# fn main() {
    use embed_it::{Content, EntryPath, Meta, Entries, Entry};
    assert_eq!(Assets.hello_txt().as_str(), &Some("hello"));
    assert_eq!(Assets.one_txt_1().as_str(), &Some("one"));
    assert_eq!(Assets.world_txt().as_str(), &Some("world"));

    assert_eq!(Assets.one_txt().hello().content(), b"hello");
    assert_eq!(Assets.one_txt().world().content(), b"world");

    assert_eq!(Assets.one_txt().children(), &vec!["one_txt/hello", "one_txt/world"]);

    let entries: &'static [Entry<_, _>] = Assets.entries();
    for entry in entries {
        println!("relative_path: {:?}", entry.map(|d| d.path(), |f| f.path()).value().relative_path_str());
        println!("{:?}", entry.map(|d| d.metadata(), |f| f.metadata()).value());
        println!("{:#?}", entry);
    }

# }
```

## How does fs-entry's name turn into Rust identifiers?
Each name will be processed and any unsuitable symbol will be replaced with `_`. This might cause a problem with a level uniqueness of identifiers, for example, all of the entry names below turn into `one_txt`.
- one+txt
- one-txt
- one_txt

The macro handles this problem and generates methods with a numeric suffix. In that case it would be 
- one+txt - `one_txt()`
- one-txt - `one_txt_1()`
- one_txt - `one_txt_2()`

Entries are sorted unambiguously by entry kind (directories first, then files) and subsequently by path.

This works for struct names in the same way

- one+txt - `OneTxt`
- one-txt - `OneTxt1`
- one_txt - `OneTxt2`


## What code will be generated by macros

1. The macro generates definitions for traits `Dir` and `File` where each is a compilation of the all derived traits
1. The macro generates definitions for traits `EntryDir` and `EntryFile` where each is a compilation of a previous step trait and the all `field` traits with `global`
1. The macro generates structs `DynDir(&'static dyn EntryDir)` and `DynFile(&'static dyn EntryFile)` which is used for dynamic dispatch (like `Entries` or `Index` traits).
1. The macro implements the intersection of the `Dir` and `File` traits for the `Entry` struct
4. The macro generates traits for `FileFieldFactory` and `DirFieldFactory` with bounds to `File`/`Dir` traits for the argument of the method
5. The macro generates traits for each `field`
6. For any entry starting from the root:
    * For each type of entry, the macro implements the requested suitable embedded traits (like `Content`, `Path`, `Metadata`, `Entries`, `Index`, etc.)
    * For each type of entry, the macro implements traits for all suitable fields from the step 6
    * For a directory, the macro recursively generates code for each child


**NOTE:** All instances are static, and this staticness is achieved 

* by `const` for any const items, like file content or file path
```rust
struct Hello;
#[automatically_derived]
impl ::embed_it::Content for Hello {
    fn content(&self) -> &'static [u8] {
        const VALUE: &[u8] = b"hello"; // in a real-world scenario, it would be `include_bytes!(...)`
        VALUE
    }
}
```

* by a `static` `LazyLock` for non-const items, which can be created without a context
```rust

use embed_it::{Entry, Index, Content};

pub struct Assets;

pub trait Dir: Send + Sync + Index<EntryDir, EntryFile> {}
pub trait File: Send + Sync + Content {}

pub struct EntryDir(&'static dyn Dir);
pub struct EntryFile(&'static dyn File);


#[automatically_derived]
impl Index<EntryDir, EntryFile> for Assets {
    fn get(&self, path: &str) -> Option<&'static Entry<EntryDir, EntryFile>> {
        static VALUE: ::std::sync::LazyLock<
            ::std::collections::HashMap<
                &'static str,
                Entry<EntryDir, EntryFile>,
            >,
        > = ::std::sync::LazyLock::new(|| {
            let mut map = ::std::collections::HashMap::with_capacity(2usize);
            // inserts
            map
        });
        VALUE.get(path)
    }

}

```

* by a `static` `OnceLock` for non-const items, which require a context (like additional `field`s)

```rust

// user-defined struct and implementation
pub struct AsStr;
impl FileFieldFactory for AsStr {
    type Field = Option<&'static str>;
    fn create<T: File + ?Sized>(data: &T) -> Self::Field {
        std::str::from_utf8(data.content()).ok()
    }
}

pub struct Assets;

// auto-generated
pub trait Dir: Send + Sync {}
pub trait File: Send + Sync + ::embed_it::Content {}

pub struct Hello;
impl ::embed_it::Content for Hello {
    fn content(&self) -> &'static [u8] {
        // Some implementation
        unimplemented!();
    }
}

impl File for Hello {};

pub enum Entry {
    Dir(&'static dyn Dir),
    File(&'static dyn File),
}

pub trait FileFieldFactory {
    type Field;
    fn create<T: File + ?Sized>(data: &T) -> Self::Field;
}

pub trait AsStrField {
    fn as_str(
        &self,
    ) -> &'static <AsStr as FileFieldFactory>::Field;
}

#[automatically_derived]
impl AsStrField for Hello {
    fn as_str(
        &self,
    ) -> &'static <AsStr as FileFieldFactory>::Field {
        static VALUE: ::std::sync::OnceLock<
            <AsStr as FileFieldFactory>::Field,
        > = ::std::sync::OnceLock::new();
        VALUE.get_or_init(|| {
            <AsStr as FileFieldFactory>::create(self)
        })
    }

}

```