use crate::IdentError;
use std::fmt::{Display, Formatter};

use crate::util::is_ident_compatible;

pub enum PgIdent<T> where T: AsRef<str> {
    Id(T),
    Quoted(String)
}

impl<T: AsRef<str>> PgIdent<T> {
    pub fn new(id: T) -> Result<Self, IdentError> {
        if is_ident_compatible(id.as_ref()) {
            Ok(Self::Id(id))
        } else {
            let id = id.as_ref();
            if id.contains('\x00') {
                return Err(IdentError::NullByteError{});
            }
            Ok(Self::Quoted(id.replace("\"", "\"\"")))
        }
    }
}

impl<T> Display for PgIdent<T> where T: AsRef<str> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            Self::Id(id) => write!(f, "{}", id.as_ref()),
            Self::Quoted(id) => write!(f, "\"{}\"", id),
        }
    }
}

impl TryFrom<&str> for PgIdent<String> {
    type Error = IdentError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PgIdent::new(value.to_string())
    }
}

impl<T> Clone for PgIdent<T> where T: AsRef<str> + Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Id(id) => Self::Id(id.clone()),
            Self::Quoted(id) => Self::Quoted(id.clone()),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let id: PgIdent<String> = "foo".try_into().unwrap();
        assert_eq!(format!("{}", id), "foo");
    }

    #[test]
    fn simple() {
        let id = PgIdent::new("foo").unwrap();
        assert_eq!(format!("{}", id), "foo");
    }

    #[test]
    fn quoted() {
        let id = PgIdent::new("FOO").unwrap();
        assert!(matches!(id, PgIdent::Quoted(_)));
        assert_eq!(format!("{}", id), "\"FOO\"");
    }

    #[test]
    fn quoted_escaped() {
        let id = PgIdent::new("The \"table\"").unwrap();
        assert!(matches!(id, PgIdent::Quoted(_)));
        assert_eq!(format!("{}", id), "\"The \"\"table\"\"\"");
    }

}