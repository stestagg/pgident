
pub fn is_ident_compatible(id: &str) -> bool {
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