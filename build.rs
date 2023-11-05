use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Read version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    // Split the version into major, minor, and patch
    let version_parts: Vec<u32> = version
        .split('.')
        .map(|p| {
            p.parse::<u32>().expect(&format!(
                "Version part can't be parsed into a u32: \"{}\"",
                p
            ))
        })
        .collect();
    let (major, minor, patch) = match &version_parts[..] {
        [major, minor, patch] => (major, minor, patch),
        _ => panic!("Invalid version format"),
    };

    // Write the parsed version parts to a file that will be included in the build
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version");
    let mut f = File::create(&dest_path).unwrap();

    writeln!(
        f,
        "({}, {}, {})",
        major, minor, patch
    )
    .unwrap();
}
