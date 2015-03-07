#[derive(Debug, PartialEq)]
pub enum Type {
    JSON
}

pub fn from_path(path: &Path) -> Option<Type> {
    match path.filename_str() {
        Some(filename) => {
            let extension = filename.split('.').last();
            match extension {
                Some("json") => Some(Type::JSON),
                _ => None,
            }
        },
        None => return None,
    }
}

mod tests {
    use super::from_path;
    use super::Type;

    #[test]
    fn from_path_works() {
        let buffer_type = from_path(&Path::new("file.json")).unwrap();
        assert_eq!(buffer_type, Type::JSON);
    }
}
