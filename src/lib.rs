
pub mod error;
pub mod name;
pub mod ident;
mod util;

pub use error::IdentError;
pub use name::PgName;
pub use ident::PgIdent;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespaced_quoted() {
        let id = PgName::new_ns(vec!["foo", "FOO"]).unwrap();
        assert!(matches!(id, PgName::IdPair(_, _)));
        assert_eq!(format!("{}", id), "foo.\"FOO\"");
    }

    #[test]
    fn test_nested_dots() {
        let id = PgName::new_ns(vec!["foo.foo", "FOO.FOO"]).unwrap();
        assert!(matches!(id, PgName::IdPair(_, _)));
        assert_eq!(format!("{}", id), "\"foo.foo\".\"FOO.FOO\"");
    }
    
}
