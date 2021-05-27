pub(crate) const RUST_KEYWORDS: &[&str] = &[
    "as", "use", "break", "const", "continue", "crate", "else", "if", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "for", "let", "loop",
    "match", "mod", "move", "mut", "pub", "impl", "ref", "return", "Self",
    "self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "abstract", "alignof", "become", "box", "do",
    "final", "macro", "offsetof", "override", "priv", "proc", "pure", "sizeof",
    "typeof", "unsized", "virtual", "yield",
];

pub(crate) fn get_mod_name(name: &str) -> proc_macro2::Ident {
    callsite_ident(&if RUST_KEYWORDS.contains(&name) {
        format!("{}_mod", name)
    } else {
        name.to_string()
    })
}

pub(crate) fn callsite_ident(name: &str) -> proc_macro2::Ident {
    assert!(!RUST_KEYWORDS.contains(&name));
    proc_macro2::Ident::new(name, proc_macro2::Span::call_site())
}
pub(crate) fn under_to_camel(name: &str) -> String {
    name.split('_').map(|x| capitalize_first_char(x)).collect()
}

pub(crate) fn camel_to_under(name: &str) -> String {
    name.chars()
        .fold(vec![String::new()], |mut v, c| {
            if c.is_ascii_uppercase() {
                v.push(c.to_ascii_lowercase().to_string());
                v
            } else {
                let end = v.len() - 1;
                v[end].push(c);
                v
            }
        })
        .into_iter()
        .skip_while(String::is_empty)
        .collect::<Vec<String>>()
        .join("_")
}
fn capitalize_first_char(input: &str) -> String {
    if input.len() == 0 {
        dbg!(input);
        return input.to_string();
    }
    let mut ret = input.to_string();
    let ch = ret.remove(0);
    ret.insert(0, ch.to_ascii_uppercase());
    ret
}
