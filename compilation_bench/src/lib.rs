pub fn print_entries_count(count: usize) {
    println!("###ENTRIES_COUNT###: {count}")
}

#[cfg(feature = "bench-files")]
mod bench_files {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_FILES")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "bench-dirs")]
mod bench_dirs {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_DIRS")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "bench-nested-dirs")]
mod bench_nested_dirs {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_NESTED_DIRS")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "bench-nesting")]
mod bench_nesting {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_NESTING")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "bench-lots-of-files")]
mod bench_lots_of_files {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_LOTS_OF_FILES")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "bench-lots-of-dirs")]
mod bench_lots_of_dirs {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_LOTS_OF_DIRS")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "bench-lots-of-nesting-items")]
mod bench_lots_of_nesting_items {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$BENCH_LOTS_OF_NESTING_ITEMS")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}

#[cfg(feature = "rust-analyzer")]
mod rust_analyzer {
    use embed_it::Embed;

    #[derive(Embed)]
    #[embed(path = "$RUST_ANALYZER")]
    pub struct Assets;

    #[test]
    fn test() {
        use embed_it::RecursiveChildCount;
        super::print_entries_count(Assets.recursive_child_count());
    }
}
