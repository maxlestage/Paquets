//! A small syntax highlighter: Rust (and JSON), plus a light one for shell,
//! TOML, YAML and Dockerfiles. Shared by the home page and the docs.

/// Token categories (each one has its CSS class).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Plain,
    Keyword,
    String,
    Comment,
    Number,
    Macro,
    Type,
    Function,
    Punct,
    Lifetime,
}

impl Kind {
    pub fn class(self) -> &'static str {
        match self {
            Kind::Plain => "t-plain",
            Kind::Keyword => "t-kw",
            Kind::String => "t-str",
            Kind::Comment => "t-com",
            Kind::Number => "t-num",
            Kind::Macro => "t-mac",
            Kind::Type => "t-type",
            Kind::Function => "t-fn",
            Kind::Punct => "t-punct",
            Kind::Lifetime => "t-life",
        }
    }
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "Some", "None", "Ok", "Err", "null",
];

pub type Tokens = Vec<(Kind, String)>;

/// Splits Rust `code` into colored tokens.
pub fn tokenize(code: &str) -> Tokens {
    let chars: Vec<char> = code.chars().collect();
    let mut out: Tokens = Vec::new();
    let mut push = |kind: Kind, text: String| match out.last_mut() {
        Some((k, t)) if *k == kind => t.push_str(&text),
        _ => out.push((kind, text)),
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Line comment.
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            push(Kind::Comment, chars[start..i].iter().collect());
            continue;
        }
        // Block comment.
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            let start = i;
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            push(Kind::Comment, chars[start..i].iter().collect());
            continue;
        }
        // Strings.
        if c == '"' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(chars.len());
            push(Kind::String, chars[start..i].iter().collect());
            continue;
        }
        // Raw strings r#"..."#.
        if c == 'r' && chars.get(i + 1) == Some(&'#') && chars.get(i + 2) == Some(&'"') {
            let start = i;
            i += 3;
            while i + 1 < chars.len() && !(chars[i] == '"' && chars[i + 1] == '#') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            push(Kind::String, chars[start..i].iter().collect());
            continue;
        }
        // Characters ('a', '\n') and lifetimes ('static).
        if c == '\'' {
            if chars.get(i + 2) == Some(&'\'') {
                push(Kind::String, chars[i..i + 3].iter().collect());
                i += 3;
                continue;
            }
            if chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\'') {
                push(Kind::String, chars[i..i + 4].iter().collect());
                i += 4;
                continue;
            }
            if chars.get(i + 1).is_some_and(|c| c.is_alphabetic() || *c == '_') {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                push(Kind::Lifetime, chars[start..i].iter().collect());
                continue;
            }
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric()
                    || chars[i] == '_'
                    || (chars[i] == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
            {
                i += 1;
            }
            push(Kind::Number, chars[start..i].iter().collect());
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if chars.get(i) == Some(&'!') && chars.get(i + 1) != Some(&'=') {
                i += 1;
                push(Kind::Macro, format!("{word}!"));
                continue;
            }
            let kind = if KEYWORDS.contains(&word.as_str()) {
                Kind::Keyword
            } else if word.chars().next().is_some_and(char::is_uppercase) {
                Kind::Type
            } else if chars.get(i) == Some(&'(')
                || (chars.get(i) == Some(&':')
                    && chars.get(i + 1) == Some(&':')
                    && chars.get(i + 2) == Some(&'<'))
            {
                Kind::Function
            } else {
                Kind::Plain
            };
            push(kind, word);
            continue;
        }
        if c == '=' && chars.get(i + 1) == Some(&'>') {
            push(Kind::Keyword, "=>".into());
            i += 2;
            continue;
        }
        if c == '#' && (chars.get(i + 1) == Some(&'[') || chars.get(i + 1) == Some(&'!')) {
            let start = i;
            let mut depth = 0;
            while i < chars.len() {
                match chars[i] {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    '\n' => break,
                    _ => {}
                }
                i += 1;
            }
            i = (i + 1).min(chars.len());
            push(Kind::Macro, chars[start..i].iter().collect());
            continue;
        }
        let kind = if "{}()[];,.:<>|&?=!+-*/%".contains(c) {
            Kind::Punct
        } else {
            Kind::Plain
        };
        push(kind, c.to_string());
        i += 1;
    }
    out
}

/// A light highlighter for shell, TOML, YAML, Dockerfile…: `#` comments,
/// strings, the `$` prompt, `[sections]` and Dockerfile instructions.
pub fn tokenize_simple(code: &str, language: &str) -> Tokens {
    let mut out: Tokens = Vec::new();
    let push = |out: &mut Tokens, kind: Kind, text: &str| {
        if !text.is_empty() {
            out.push((kind, text.to_owned()));
        }
    };
    let shell = matches!(language, "sh" | "bash" | "shell" | "console" | "zsh");
    for (n, line) in code.split('\n').enumerate() {
        if n > 0 {
            push(&mut out, Kind::Plain, "\n");
        }
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        push(&mut out, Kind::Plain, indent);
        let mut rest = trimmed;
        if matches!(language, "toml" | "ini") && rest.starts_with('[') {
            push(&mut out, Kind::Type, rest);
            continue;
        }
        if shell && rest.starts_with("$ ") {
            push(&mut out, Kind::Punct, "$ ");
            rest = &rest[2..];
            // The command itself.
            let end = rest.find(' ').unwrap_or(rest.len());
            push(&mut out, Kind::Function, &rest[..end]);
            rest = &rest[end..];
        }
        if language == "dockerfile" {
            let end = rest.find(' ').unwrap_or(rest.len());
            let word = &rest[..end];
            if !word.is_empty() && word.chars().all(|c| c.is_ascii_uppercase()) {
                push(&mut out, Kind::Keyword, word);
                rest = &rest[end..];
            }
        }
        if matches!(language, "toml" | "yaml" | "yml" | "ini")
            && let Some(eq) = rest.find(['=', ':'])
            && !rest.starts_with('#')
            && !rest.starts_with('"')
        {
            let key = &rest[..eq];
            if !key.trim().is_empty() && !key.contains('"') && !key.trim().contains(' ') {
                push(&mut out, Kind::Function, key);
                rest = &rest[eq..];
            }
        }
        let mut plain_start = 0;
        let mut chars = rest.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            let starts_comment = c == '#' && (i == 0 || rest[..i].ends_with(char::is_whitespace));
            if starts_comment {
                push(&mut out, Kind::Plain, &rest[plain_start..i]);
                push(&mut out, Kind::Comment, &rest[i..]);
                plain_start = rest.len();
                break;
            }
            if c == '"' || c == '\'' {
                if let Some(end) = rest[i + 1..].find(c).map(|j| i + 1 + j + 1) {
                    push(&mut out, Kind::Plain, &rest[plain_start..i]);
                    push(&mut out, Kind::String, &rest[i..end]);
                    plain_start = end;
                    while chars.peek().is_some_and(|&(j, _)| j < end) {
                        chars.next();
                    }
                }
            }
        }
        push(&mut out, Kind::Plain, &rest[plain_start.min(rest.len())..]);
    }
    out
}

/// Highlights `code` written in `language` (the info string of a fenced
/// block). `None` for plain text.
pub fn highlight(code: &str, language: &str) -> Option<Tokens> {
    match language {
        "rust" | "rs" | "json" | "js" | "javascript" | "ts" => Some(tokenize(code)),
        "" | "text" | "txt" | "plain" => None,
        other => Some(tokenize_simple(code, other)),
    }
}

/// Keeps only the first `limit` characters of the tokens.
pub fn truncate(tokens: &[(Kind, String)], limit: usize) -> Tokens {
    let mut left = limit;
    let mut out = Vec::new();
    for (kind, text) in tokens {
        if left == 0 {
            break;
        }
        let n = text.chars().count();
        if n <= left {
            out.push((*kind, text.clone()));
            left -= n;
        } else {
            out.push((*kind, text.chars().take(left).collect()));
            left = 0;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(tokens: &Tokens, kind: Kind) -> Vec<&str> {
        tokens
            .iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, t)| t.as_str())
            .collect()
    }

    #[test]
    fn highlights_rust() {
        let t = tokenize("#[model]\nstruct Todo { id: i64 }\nlet s: &'static str = \"hi\"; // ok\nprintln!(\"{}\", 1.5);");
        assert_eq!(kinds(&t, Kind::Macro), ["#[model]", "println!"]);
        assert!(kinds(&t, Kind::Keyword).contains(&"struct"));
        assert_eq!(kinds(&t, Kind::Type), ["Todo"]);
        assert_eq!(kinds(&t, Kind::Lifetime), ["'static"]);
        assert_eq!(kinds(&t, Kind::Comment), ["// ok"]);
        assert_eq!(kinds(&t, Kind::Number), ["1.5"]);
        // Nothing is lost.
        let text: String = t.iter().map(|(_, s)| s.as_str()).collect();
        assert!(text.ends_with("println!(\"{}\", 1.5);"));
    }

    #[test]
    fn highlights_turbofish_and_chars() {
        let t = tokenize("app.api::<Todo>(\"/api\"); let c = 'x';");
        assert!(kinds(&t, Kind::Function).contains(&"api"));
        assert!(kinds(&t, Kind::String).contains(&"'x'"));
    }

    #[test]
    fn highlights_shell_and_toml() {
        let t = tokenize_simple("$ cargo install paquets # the CLI", "bash");
        assert_eq!(kinds(&t, Kind::Function), ["cargo"]);
        assert_eq!(kinds(&t, Kind::Comment), ["# the CLI"]);
        let t = tokenize_simple("[dependencies]\npaquets = \"0.1\"", "toml");
        assert_eq!(kinds(&t, Kind::Type), ["[dependencies]"]);
        assert_eq!(kinds(&t, Kind::String), ["\"0.1\""]);
        assert_eq!(kinds(&t, Kind::Function), ["paquets "]);
    }

    #[test]
    fn truncates_tokens() {
        let t = tokenize("let x = 1;");
        let cut: String = truncate(&t, 5).into_iter().map(|(_, s)| s).collect();
        assert_eq!(cut, "let x");
    }
}
