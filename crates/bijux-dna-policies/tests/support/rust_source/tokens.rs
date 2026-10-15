#![cfg(test)]

pub(crate) struct SourceToken<'a> {
    pub(crate) text: &'a str,
    pub(crate) literal: bool,
}

fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> Option<usize> {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
        } else if bytes[index] == quote {
            return Some(index + 1);
        } else {
            index += 1;
        }
    }
    None
}

enum RawLiteralEnd {
    NotRaw,
    Closed(usize),
    Unterminated,
}

fn raw_literal_end(bytes: &[u8], start: usize) -> RawLiteralEnd {
    let mut quote = start + 1;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return RawLiteralEnd::NotRaw;
    }
    let hashes = quote - start - 1;
    let mut index = quote + 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && (0..hashes).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'))
        {
            return RawLiteralEnd::Closed(index + hashes + 1);
        }
        index += 1;
    }
    RawLiteralEnd::Unterminated
}

fn character_literal_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut end = start + 1;
    if bytes.get(end) == Some(&b'\\') {
        end += 1;
        match bytes.get(end)? {
            b'x' => end += 3,
            b'u' if bytes.get(end + 1) == Some(&b'{') => {
                end += 2;
                while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit() || *byte == b'_') {
                    end += 1;
                }
                if bytes.get(end) != Some(&b'}') {
                    return None;
                }
                end += 1;
            }
            _ => end += 1,
        }
    } else {
        end += content.get(end..)?.chars().next()?.len_utf8();
    }
    (bytes.get(end) == Some(&b'\'')).then_some(end + 1)
}

// This bounded source scan preserves literal bytes and complete argument boundaries;
// runtime resolver contracts remain responsible for executable provenance.
pub(crate) fn source_tokens(content: &str) -> Option<Vec<SourceToken<'_>>> {
    let bytes = content.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        let character = content.get(index..)?.chars().next()?;
        if character.is_whitespace() {
            index += character.len_utf8();
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let mut depth = 1_usize;
            index += 2;
            while depth > 0 {
                match bytes.get(index..index + 2)? {
                    b"/*" => {
                        depth = depth.checked_add(1)?;
                        index += 2;
                    }
                    b"*/" => {
                        depth -= 1;
                        index += 2;
                    }
                    _ => index += 1,
                }
            }
            continue;
        }
        let prefix = if matches!(bytes[index], b'b' | b'c') { index + 1 } else { index };
        let raw = if bytes.get(prefix) == Some(&b'r') {
            match raw_literal_end(bytes, prefix) {
                RawLiteralEnd::NotRaw => None,
                RawLiteralEnd::Closed(end) => Some(end),
                RawLiteralEnd::Unterminated => return None,
            }
        } else {
            None
        };
        let literal = raw.or_else(|| match bytes.get(prefix) {
            Some(b'"') => quoted_end(bytes, prefix, b'"'),
            Some(b'\'') => character_literal_end(content, prefix),
            _ => None,
        });
        if let Some(end) = literal {
            index = end;
            tokens.push(SourceToken { text: content.get(start..end)?, literal: true });
            continue;
        }
        if bytes[index] == b'"' || bytes.get(prefix) == Some(&b'"') {
            return None;
        }
        if bytes.get(index..index + 2) == Some(b"::") {
            index += 2;
        } else if character.is_alphanumeric() || character == '_' {
            index += character.len_utf8();
            while let Some(character) = content.get(index..)?.chars().next() {
                if !character.is_alphanumeric() && character != '_' {
                    break;
                }
                index += character.len_utf8();
            }
        } else {
            index += character.len_utf8();
        }
        tokens.push(SourceToken { text: content.get(start..index)?, literal: false });
    }
    Some(tokens)
}
