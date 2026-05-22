fn main() {
    rhusky::Rhusky::new().hooks_dir(".githooks").install().ok();
}
