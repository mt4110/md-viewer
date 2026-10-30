fn main() {
    let path = std::env::args_os().nth(1).expect("Markdown path");
    let document = md_viewer::document::load(std::path::Path::new(&path)).expect("read document");
    println!("{}", serde_json::to_string(&document).unwrap());
}
