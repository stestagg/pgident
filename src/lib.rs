use std::fmt::{Display, Formatter};
use std::vec::Vec;
use std::error::Error;

#[derive(Debug)]
pub struct NullByteError{}
impl Error for NullByteError {}

impl Display for NullByteError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Null byte in identifier")
    }
}

pub enum PgIdent<T> where T: AsRef<str> {
    Id(T),
    Quoted(String)
}


fn is_ident_compatible(id: &str) -> bool {
    // Rules taken from: https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
    // "The system uses no more than NAMEDATALEN-1 bytes of an identifier; longer names can be written
    // in commands, but they will be truncated. By default, NAMEDATALEN is 64 so the maximum identifier
    // length is 63 bytes."
    // As a practical balance, we'll assume 63 bytes default is kept, and disallow longer
    if id.len() > 63 || id.len() == 0 {
        return false;
    }

    let mut char_it = id.chars();
    let first = char_it.next();
    // SQL identifiers and key words must begin with a letter 
    // (a-z, but also letters with diacritical marks and non-Latin letters)
    // or an underscore (_). 
    // Because IDs are case-insensitive, and postgresql defaults to lower-case
    // we only allow lower-case here..
    match first {
        Some(c) => {
            if !c.is_lowercase() && c != '_' {
                return false;
            }
        }
        None => return false,
    }
    // Subsequent characters in an identifier or key word can be 
    // letters, underscores, digits (0-9), or dollar signs ($).
    char_it.all(|c| c.is_lowercase() || c.is_numeric() || c == '_' || c == '$')
}

impl<T> PgIdent<T> where T: AsRef<str> {
    fn new(id: T) -> Result<Self, NullByteError> {
        if is_ident_compatible(id.as_ref()) {
            Ok(Self::Id(id))
        } else {
            let id = id.as_ref();
            if id.contains('\x00') {
                return Err(NullByteError{});
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


pub enum PgName<T> where T: AsRef<str> {
    Id(PgIdent<T>),
    IdPair(PgIdent<T>, PgIdent<T>),
    Namespaced(Vec<PgIdent<T>>),
}

impl <T: AsRef<str>> PgName<T> {
    fn new(id: T) -> Result<Self, NullByteError> {
        Ok(Self::Id(PgIdent::new(id)?))
    }

    fn new_ns<U>(ns: U) -> Result<Self, NullByteError> 
    where U: IntoIterator<Item=T>
    {
        let mut ids = Vec::new();
        for id in ns {
            ids.push(PgIdent::new(id)?);
        }
        if ids.len() == 1 {
            return Ok(Self::Id(ids.pop().unwrap()));
        }
        if ids.len() == 2 {
            return Ok(Self::IdPair(ids.remove(0), ids.remove(0)));
        }
        Ok(Self::Namespaced(ids))
    }
}

impl<T: AsRef<str>> Display for PgName<T> {

    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            Self::Id(id) => write!(f, "{}", id),
            Self::IdPair(schema, table) => write!(f, "{}.{}", schema, table),
            Self::Namespaced(ids) => {
                let mut first = true;
                for id in ids {
                    if !first {
                        write!(f, ".")?;
                    }
                    first = false;
                    write!(f, "{}", id)?;
                }
                Ok(())
            }
        }
    }
}

pub fn name<T: AsRef<str>>(id: T) -> Result<PgName<T>, NullByteError> {
    PgName::new(id)
}

pub fn pair<T: AsRef<str>>(schema: T, table: T) -> Result<PgName<T>, NullByteError> {
    let schema = PgIdent::new(schema)?;
    let table = PgIdent::new(table)?;
    Ok(PgName::IdPair(schema, table))
}

pub fn namespaced<T, U>(ns: U) -> Result<PgName<T>, NullByteError> 
where T: AsRef<str>,
      U: IntoIterator<Item=T>
{
    PgName::new_ns(ns)
}


#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_name_single() {
        let id = namespaced(vec!["foo"]).unwrap();
        assert!(matches!(id, PgName::Id(_)));
        assert_eq!(format!("{}", id), "foo");
    }

    #[test]
    fn test_namespaced() {
        let id = namespaced(vec!["foo", "bar"]).unwrap();
        assert!(matches!(id, PgName::IdPair(_, _)));
        assert_eq!(format!("{}", id), "foo.bar");
    }

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
