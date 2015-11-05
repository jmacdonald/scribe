use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Type {
    CoffeeScript,
    JavaScript,
    JSON,
    XML,
    Ruby,
    Rust,
    ERB
}

pub fn from_path(path: &Path) -> Option<Type> {
    match path.to_str() {
        Some(filename) => {
            let extension = filename.split('.').last();
            match extension {
                Some("coffee") => Some(Type::CoffeeScript),
                Some("js") => Some(Type::JavaScript),
                Some("json") => Some(Type::JSON),
                Some("xml") => Some(Type::XML),
                Some("rake") => Some(Type::Ruby),
                Some("rb") => Some(Type::Ruby),
                Some("rs") => Some(Type::Rust),
                Some("erb") => Some(Type::ERB),
                _ => None,
            }
        },
        None => return None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::from_path;
    use super::Type;

    #[test]
    fn from_path_works() {
        let buffer_type = from_path(&Path::new("file.json")).unwrap();
        assert_eq!(buffer_type, Type::JSON);
    }
}
