use quote::format_ident;

pub fn escape_ident(name: &str) -> syn::Ident {
    match name {
        " break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false" | "fn"
        | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" | "trait"
        | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await" | "dyn" => {
            format_ident!("r#{}", name)
        }
        _ => format_ident!("{}", name),
    }
}
