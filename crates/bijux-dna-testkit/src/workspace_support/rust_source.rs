#[derive(Default)]
struct LexicalState {
    block_comment_depth: usize,
    in_string: bool,
    in_char: bool,
    raw_string_hashes: Option<usize>,
    escaped: bool,
}

/// Remove inline modules governed by `#[cfg(test)]` while preserving production source text.
///
/// This helper is intended for repository policy scans that inspect source lexically rather than
/// compiling it. It ignores braces inside comments, strings, and character literals when finding
/// the end of a test module.
#[must_use]
pub fn strip_rust_test_modules(raw: &str) -> String {
    let mut retained = Vec::new();
    let mut pending_attribute_lines = Vec::new();
    let mut awaiting_test_module = false;
    let mut skip_depth = 0usize;
    let mut lexical_state = LexicalState::default();

    for line in raw.lines() {
        let trimmed = line.trim();
        if skip_depth > 0 {
            skip_depth = apply_brace_delta(line, skip_depth, &mut lexical_state);
            continue;
        }

        if is_test_cfg_attribute(trimmed) {
            awaiting_test_module = true;
            pending_attribute_lines.push(line);
            continue;
        }

        if awaiting_test_module {
            if trimmed.is_empty() || trimmed.starts_with("#[") {
                pending_attribute_lines.push(line);
                continue;
            }
            if trimmed.starts_with("mod ") && line.contains('{') {
                lexical_state = LexicalState::default();
                skip_depth = apply_brace_delta(line, 0, &mut lexical_state);
                pending_attribute_lines.clear();
                awaiting_test_module = false;
                continue;
            }
            retained.append(&mut pending_attribute_lines);
            awaiting_test_module = false;
        }

        retained.push(line);
    }

    retained.append(&mut pending_attribute_lines);
    retained.join("\n")
}

fn is_test_cfg_attribute(trimmed: &str) -> bool {
    trimmed.chars().filter(|character| !character.is_whitespace()).collect::<String>()
        == "#[cfg(test)]"
}

fn apply_brace_delta(line: &str, initial_depth: usize, state: &mut LexicalState) -> usize {
    let bytes = line.as_bytes();
    let mut depth = initial_depth;
    let mut index = 0usize;
    while index < bytes.len() {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();

        if let Some(hash_count) = state.raw_string_hashes {
            if current == b'"'
                && (0..hash_count).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'))
            {
                state.raw_string_hashes = None;
                index += hash_count + 1;
            } else {
                index += 1;
            }
            continue;
        }
        if state.block_comment_depth > 0 {
            if current == b'/' && next == Some(b'*') {
                state.block_comment_depth += 1;
                index += 2;
                continue;
            }
            if current == b'*' && next == Some(b'/') {
                state.block_comment_depth -= 1;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if state.in_string || state.in_char {
            if state.escaped {
                state.escaped = false;
            } else if current == b'\\' {
                state.escaped = true;
            } else if (state.in_string && current == b'"') || (state.in_char && current == b'\'') {
                state.in_string = false;
                state.in_char = false;
            }
            index += 1;
            continue;
        }
        if current == b'/' && next == Some(b'/') {
            break;
        }
        if current == b'/' && next == Some(b'*') {
            state.block_comment_depth = 1;
            index += 2;
            continue;
        }
        if current == b'r' {
            let mut quote_index = index + 1;
            while bytes.get(quote_index) == Some(&b'#') {
                quote_index += 1;
            }
            if bytes.get(quote_index) == Some(&b'"') {
                state.raw_string_hashes = Some(quote_index - index - 1);
                index = quote_index + 1;
                continue;
            }
        }
        if current == b'"' {
            state.in_string = true;
        } else if current == b'\'' && has_char_literal_terminator(bytes, index + 1) {
            state.in_char = true;
        } else if current == b'{' {
            depth += 1;
        } else if current == b'}' {
            depth = depth.saturating_sub(1);
        }
        index += 1;
    }
    depth
}

fn has_char_literal_terminator(bytes: &[u8], mut index: usize) -> bool {
    let mut escaped = false;
    while let Some(current) = bytes.get(index).copied() {
        if current.is_ascii_whitespace() {
            return false;
        }
        if escaped {
            escaped = false;
        } else if current == b'\\' {
            escaped = true;
        } else if current == b'\'' {
            return true;
        }
        index += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::strip_rust_test_modules;

    #[test]
    fn strips_attributed_nested_test_modules_without_counting_literal_braces() {
        let source = r##"pub fn production() -> &'static str { "kept" }

#[cfg ( test )]
#[allow(clippy::needless_raw_string_hashes)]
mod tests {
    fn nested() {
        if true {
            let text = "a } brace";
            let character = '{';
            let raw = r#"a \"quoted\" } brace"#;
            /* a comment with } and a nested /* { */ comment */
            assert_eq!(text.len() + raw.len(), 27);
        }
    }
}

pub fn after() -> usize { 2 }
"##;

        let stripped = strip_rust_test_modules(source);
        assert!(stripped.contains("pub fn production"));
        assert!(stripped.contains("pub fn after"));
        assert!(!stripped.contains("mod tests"));
        assert!(!stripped.contains("a } brace"));
    }

    #[test]
    fn preserves_cfg_attributes_that_do_not_govern_inline_test_modules() {
        let source = "#[cfg(test)]\npub fn test_support() {}\n#[cfg(unix)]\nmod unix {}";
        assert_eq!(strip_rust_test_modules(source), source);
    }
}
