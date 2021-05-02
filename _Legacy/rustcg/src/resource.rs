use std::path::Path;

pub fn load_bytes(name: &str) -> Vec<u8> {
    let path = Path::new("./assets/").join(name);
    return std::fs::read(path).unwrap();
}
