use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=runtime/frizzi");

    let root = Path::new("runtime/frizzi");
    let mut files = Vec::new();
    if root.join("typst").is_file() {
        files.push(root.join("typst"));
        for family in fs::read_dir(root.join("fonts")).expect("bundled PDF fonts") {
            for file in fs::read_dir(family.unwrap().path()).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_some_and(|ext| ext == "ttf") {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    let mut bundled = String::from("&[");
    for path in files {
        let name = path.strip_prefix(root).unwrap().to_str().unwrap();
        let source = fs::canonicalize(&path).unwrap();
        bundled.push_str(&format!("({name:?}, include_bytes!({source:?})),"));
    }
    bundled.push(']');
    let output = std::path::PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(output.join("pdf_assets.rs"), bundled).unwrap();
}
