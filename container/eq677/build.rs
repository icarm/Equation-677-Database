use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("db_sources.rs");
    fs::write(
        &dest,
        "pub static DB_SOURCES: &[((usize, usize), &str)] = &[];",
    )
    .unwrap();
}
