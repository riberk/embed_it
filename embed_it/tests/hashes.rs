#[cfg(feature = "md5")]
pub mod md5 {
    use embed_it::Md5Hash;
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        dir(derive(Md5)),
        file(derive(Md5))
    )]
    pub struct Assets;

    #[test]
    fn check_hashes() {
        assert_eq!(
            Assets.hello().md5(),
            &hex!("5d41402abc4b2a76b9719d911017c592")
        );
        assert_eq!(
            Assets.world().md5(),
            &hex!("7d793037a0760186574b0282f2f435e7")
        );
        assert_eq!(
            Assets.one().md5(),
            &hex!("f97c5d29941bfb1b2fdab0874906ab82")
        );
        assert_eq!(
            Assets.one_txt().hello().md5(),
            &hex!("5d41402abc4b2a76b9719d911017c592")
        );
        assert_eq!(
            Assets.one_txt().world().md5(),
            &hex!("7d793037a0760186574b0282f2f435e7")
        );

        assert_eq!(
            Assets.one_txt().md5(),
            &hex!("892433f329afa614006bb375a9c82859")
        );

        assert_eq!(Assets.md5(), &hex!("56e71a41c76b1544c52477adf4c8e2f7"));
    }

    mod only_dir {
        use embed_it::Md5Hash;
        use hex_literal::hex;

        #[derive(embed_it::Embed)]
        #[embed(path = "$CARGO_MANIFEST_DIR/../example_dirs/assets", dir(derive(Md5)))]
        pub struct Assets;

        #[test]
        fn only_dir() {
            assert_eq!(
                Assets.one_txt().md5(),
                &hex!("fc5e038d38a57032085441e7fe7010b0")
            );
            assert_eq!(Assets.md5(), &hex!("32d50ea62bf617bff2efe27f57b7aebf"));
        }
    }

    mod only_file {
        use embed_it::Md5Hash;
        use hex_literal::hex;

        #[derive(embed_it::Embed)]
        #[embed(path = "$CARGO_MANIFEST_DIR/../example_dirs/assets", file(derive(Md5)))]
        pub struct Assets;

        #[test]
        fn only_dir() {
            assert_eq!(
                Assets.hello().md5(),
                &hex!("5d41402abc4b2a76b9719d911017c592")
            );
        }
    }
}

#[cfg(feature = "sha1")]
pub mod sha1 {
    use embed_it::Sha1Hash;
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        dir(derive(Sha1)),
        file(derive(Sha1))
    )]
    pub struct Assets;

    #[test]
    fn check_hashes() {
        assert_eq!(
            Assets.sha1(),
            &hex!("26da80338f55108be5bcce49285a4154f6705599")
        );
    }
}

#[cfg(feature = "sha2")]
pub mod sha2 {
    use embed_it::{Sha2_224Hash, Sha2_256Hash, Sha2_384Hash, Sha2_512Hash};
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        dir(derive(Sha2_224), derive(Sha2_256), derive(Sha2_384), derive(Sha2_512),),
        file(derive(Sha2_224), derive(Sha2_256), derive(Sha2_384), derive(Sha2_512),)
    )]
    pub struct Assets;

    #[test]
    fn check_hashes() {
        assert_eq!(
            Assets.sha2_224(),
            &hex!("360c16e2d8135a337cc6ddf4134ec9cc69dd65b779db2a2807f941e4")
        );
        assert_eq!(
            Assets.sha2_256(),
            &hex!("e16b758a01129c86f871818a7b4e31c88a3c6b69d9c8319bcbc881b58f067b25")
        );
        assert_eq!(Assets.sha2_384(), &hex!("de4656a27347eee72aea1d15e85f20439673709cde5339772660bbd9d800bbde9f637eb3505f572140432625f3948175"));
        assert_eq!(Assets.sha2_512(), &hex!("bc1673b560316c6586fa1ec98ca5df3e303b66ddae944b05c71314806f88bd4b8f4c7832dfb7dd729eaca191b7142936d21bd07f750c9bc35d67f218e51bbaa4"));
    }
}

#[cfg(feature = "sha3")]
pub mod sha3 {
    use embed_it::{Sha3_224Hash, Sha3_256Hash, Sha3_384Hash, Sha3_512Hash};
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        dir(derive(Sha3_224), derive(Sha3_256), derive(Sha3_384), derive(Sha3_512),),
        file(derive(Sha3_224), derive(Sha3_256), derive(Sha3_384), derive(Sha3_512),)
    )]
    pub struct Assets;

    #[test]
    fn check_hashes() {
        assert_eq!(
            Assets.sha3_224(),
            &hex!("6949265b40fa55e0c194e3591f90e6cbf0ac100d7ed32e71d6e1e753")
        );
        assert_eq!(
            Assets.sha3_256(),
            &hex!("a2d99103dc2d1967fb05c4de99a1432e9afb1f5acc698fefb2112ce7fb9335c4")
        );
        assert_eq!(Assets.sha3_384(), &hex!("cf1f50cb53dc61b3519227887bfb20230b6878d32b10c5a9bfe016095aaecc593e612a165c89488109da62138a7214d8"));
        assert_eq!(Assets.sha3_512(), &hex!("aeff4601a53fecdad418f3245676398719d507bd7b971098ad3f4c2d495c2cc96faf022f481c0bebc0632492abd8eb9fe9f8af6d25664f33d61ff316d269682a"));
    }
}

#[cfg(feature = "blake3")]
pub mod blake3 {
    use embed_it::Blake3_256Hash;
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        dir(derive(Blake3)),
        file(derive(Blake3))
    )]
    pub struct Assets;

    #[test]
    fn check_hashes() {
        assert_eq!(
            Assets.blake3_256(),
            &hex!("b5947e2140b0fe744b1afe9a9f9031e72571c85db079413a67b4a9309f581de7")
        );
    }
}
