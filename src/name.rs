use crate::{PgIdent, IdentError};
use std::fmt::{Display, Formatter};


pub enum PgName<T> where T: AsRef<str> {
    Id(PgIdent<T>),
    IdPair(PgIdent<T>, PgIdent<T>),
    Namespaced(Vec<PgIdent<T>>),
}

impl <T: AsRef<str>> PgName<T> {

    pub fn new(id: T) -> Result<Self, IdentError> {
        Ok(Self::Id(PgIdent::new(id)?))
    }

    pub fn new_ns<U>(ns: U) -> Result<Self, IdentError> 
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

    pub fn name(&self) -> &PgIdent<T> {
        match self {
            Self::Id(id) => id,
            Self::IdPair(_, table) => table,
            Self::Namespaced(ids) => ids.last().unwrap(),
        }
    }
}

impl <T: AsRef<str> + Clone> PgName<T> {
    pub fn with_name(&self, name_part: T) -> Result<PgName<T>, IdentError> {
        let name_part = PgIdent::new(name_part)?;
        Ok(match self {
            Self::Id(_) => Self::Id(name_part),
            Self::IdPair(schema, _) => Self::IdPair(schema.clone().into(), name_part),
            Self::Namespaced(ids) => {
                let mut new_parts = ids[..ids.len()-1].to_vec();
                new_parts.push(name_part);
                Self::Namespaced(new_parts)
            }
        })
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

impl TryFrom<&str> for PgName<String> {
    type Error = IdentError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PgName::new(value.to_string())
    }
}

impl TryFrom<(&str, &str)> for PgName<String> {
    type Error = IdentError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let schema = PgIdent::new(value.0.to_string())?;
        let table = PgIdent::new(value.1.to_string())?;
        Ok(PgName::IdPair(schema, table))
    }
}

impl TryFrom<(String, String)> for PgName<String> {
    type Error = IdentError;

    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        let schema = PgIdent::new(value.0)?;
        let table = PgIdent::new(value.1)?;
        Ok(PgName::IdPair(schema, table))
    }
}

impl TryFrom<Vec<&str>> for PgName<String> {
    type Error = IdentError;

    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        PgName::new_ns(value.into_iter().map(|s| s.to_string()))
    }
}

impl TryFrom<Vec<String>> for PgName<String> {
    type Error = IdentError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        PgName::new_ns(value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let id: PgName<String> = "foo".try_into().unwrap();
        assert_eq!(format!("{}", id), "foo");
    }

    #[test]
    fn from_tuple() {
        let id: PgName<String> = ("foo", "bar").try_into().unwrap();
        assert!(matches!(id, PgName::IdPair(_, _)));
        assert_eq!(format!("{}", id), "foo.bar");
    }

    #[test]
    fn from_string_tuple() {
        let id: PgName<String> = ("foo".to_string(), "bar".to_string()).try_into().unwrap();
        assert!(matches!(id, PgName::IdPair(_, _)));
        assert_eq!(format!("{}", id), "foo.bar");
    }

    #[test]
    fn simple() {
        let id = PgName::new("foo").unwrap();
        assert_eq!(format!("{}", id), "foo");
    }

    #[test]
    fn quoted() {
        let id = PgName::new("FOO").unwrap();
        assert!(matches!(id, PgName::Id(_)));
        assert_eq!(format!("{}", id), "\"FOO\"");
    }

    #[test]
    fn quoted_escaped() {
        let id = PgName::new("The \"table\"").unwrap();
        assert!(matches!(id, PgName::Id(_)));
        assert_eq!(format!("{}", id), "\"The \"\"table\"\"\"");
    }

    #[test]
    fn test_name_id() {
        let id = PgName::new("foo").unwrap();
        assert_eq!(format!("{}", id.name()), "foo");
    }

    #[test]
    fn test_name_pair() {
        let id: PgName<String> = ("foo", "bar").try_into().unwrap();
        assert_eq!(format!("{}", id.name()), "bar");
    }

    #[test]
    fn test_name_vec() {
        let id: PgName<&'static str> = PgName::new_ns(vec!["a", "b", "c"]).unwrap();
        assert_eq!(format!("{}", id.name()), "c");
    }

}