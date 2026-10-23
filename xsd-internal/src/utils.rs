use quote::format_ident;

pub fn escape_ident(name: &str) -> syn::Ident {
    match name {
        // keywords that cannot be used as raw identifiers
        "crate" | "self" | "Self" | "super" | "_" => format_ident!("{}_", name),
        "abstract" | "as" | "async" | "await" | "become" | "box" | "break" | "const"
        | "continue" | "do" | "dyn" | "else" | "enum" | "extern" | "false" | "final" | "fn"
        | "for" | "gen" | "if" | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod"
        | "move" | "mut" | "override" | "priv" | "pub" | "ref" | "return" | "static" | "struct"
        | "trait" | "true" | "try" | "type" | "typeof" | "unsafe" | "unsized" | "use"
        | "virtual" | "where" | "while" | "yield" => {
            format_ident!("r#{}", name)
        }
        _ => format_ident!("{}", name),
    }
}

#[cfg(test)]
mod tests {
    use super::escape_ident;

    #[test]
    fn escape_keywords() {
        assert_eq!(escape_ident("break").to_string(), "r#break");
        assert_eq!(escape_ident("type").to_string(), "r#type");
        assert_eq!(escape_ident("Self").to_string(), "Self_");
        assert_eq!(escape_ident("self").to_string(), "self_");
        assert_eq!(escape_ident("name").to_string(), "name");
    }
}
