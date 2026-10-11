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
    partition_rust_test_modules(raw, false)
}

/// Select inline modules governed by `#[cfg(test)]`, excluding surrounding production code.
///
/// Uses the same lexical boundaries as [`strip_rust_test_modules`]. This is a source-policy
/// helper, not a Rust parser; it selects inline modules with their attribute on a separate line.
#[must_use]
pub fn select_rust_test_modules(raw: &str) -> String {
    partition_rust_test_modules(raw, true)
}

fn partition_rust_test_modules(raw: &str, select_tests: bool) -> String {
    let mut retained = Vec::new();
    let mut pending_attribute_lines = Vec::new();
    let mut awaiting_test_module = false;
    let mut skip_depth = 0usize;
    let mut lexical_state = LexicalState::default();

    for line in raw.lines() {
        let trimmed = line.trim();
        if skip_depth > 0 {
            if select_tests {
                retained.push(line);
            }
            skip_depth = apply_brace_delta(line, skip_depth, &mut lexical_state);
            continue;
        }

        let in_code = lexical_state.block_comment_depth == 0
            && !lexical_state.in_string
            && !lexical_state.in_char
            && lexical_state.raw_string_hashes.is_none();
        if in_code && is_test_cfg_attribute(trimmed) {
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
                if select_tests {
                    retained.append(&mut pending_attribute_lines);
                    retained.push(line);
                }
                pending_attribute_lines.clear();
                awaiting_test_module = false;
                continue;
            }
            if !select_tests {
                retained.append(&mut pending_attribute_lines);
            }
            pending_attribute_lines.clear();
            awaiting_test_module = false;
        }

        apply_brace_delta(line, 0, &mut lexical_state);
        if !select_tests {
            retained.push(line);
        }
    }

    if !select_tests {
        retained.append(&mut pending_attribute_lines);
    }
    retained.join("\n")
}

fn is_test_cfg_attribute(trimmed: &str) -> bool {
    trimmed.chars().filter(|character| !character.is_whitespace()).collect::<String>()
        == "#[cfg(test)]"
}

fn apply_brace_delta(line: &str, initial_depth: usize, state: &mut LexicalState) -> usize {
    let mut depth = initial_depth;
    visit_braces(line, state, |_, opening| {
        depth = if opening { depth + 1 } else { depth.saturating_sub(1) };
    });
    depth
}

/// Return byte ranges for braced Rust scopes, ignoring braces in literals and comments.
///
/// Unclosed scopes extend to the end of the input so lexical policy scans remain conservative.
#[must_use]
pub fn rust_block_ranges(raw: &str) -> Vec<std::ops::Range<usize>> {
    let mut state = LexicalState::default();
    let mut stack = Vec::new();
    let mut ranges = Vec::new();
    let mut offset = 0;
    for line in raw.split_inclusive('\n') {
        visit_braces(line, &mut state, |index, opening| {
            if opening {
                stack.push(offset + index);
            } else if let Some(start) = stack.pop() {
                ranges.push(start..offset + index + 1);
            }
        });
        offset += line.len();
    }
    ranges.extend(stack.into_iter().map(|start| start..raw.len()));
    ranges
}

fn visit_braces(line: &str, state: &mut LexicalState, mut visit: impl FnMut(usize, bool)) {
    let bytes = line.as_bytes();
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
            visit(index, true);
        } else if current == b'}' {
            visit(index, false);
        }
        index += 1;
    }
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
    use super::{rust_block_ranges, select_rust_test_modules, strip_rust_test_modules};

    #[test]
    fn scope_ranges_ignore_literal_braces_and_keep_nested_and_unclosed_blocks() {
        let source = "fn first() { let text = \"}\"; /* { */ if true { call(); } }\nfn second() {";
        let scopes = rust_block_ranges(source);
        let bodies = scopes.into_iter().map(|range| &source[range]).collect::<Vec<_>>();
        assert_eq!(
            bodies,
            ["{ call(); }", "{ let text = \"}\"; /* { */ if true { call(); } }", "{"]
        );
    }

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
        let selected = select_rust_test_modules(source);
        assert!(selected.contains("mod tests"));
        assert!(selected.contains("a } brace"));
        assert!(!selected.contains("pub fn production"));
        assert!(!selected.contains("pub fn after"));
    }

    #[test]
    fn preserves_cfg_attributes_that_do_not_govern_inline_test_modules() {
        let source = "#[cfg(test)]\npub fn test_support() {}\n#[cfg(unix)]\nmod unix {}";
        assert_eq!(strip_rust_test_modules(source), source);
        assert!(select_rust_test_modules(source).is_empty());
    }

    #[test]
    fn selects_each_test_module_without_intervening_production_or_literal_attributes() {
        let source = r##"const EXAMPLE: &str = r#"
#[cfg(test)]
mod fake { fn example() {} }
"#;
/*
#[cfg(test)]
mod commented { fn example() {} }
*/
#[cfg(test)]
mod first { fn owner_case() {} }
fn production_between() {}
#[cfg(test)]
mod second {
    fn other_case() { let closing = "}"; }
}
fn production_after() {}
"##;
        let selected = select_rust_test_modules(source);
        assert!(selected.contains("mod first"));
        assert!(selected.contains("mod second"));
        for excluded in
            ["EXAMPLE", "mod fake", "mod commented", "production_between", "production_after"]
        {
            assert!(!selected.contains(excluded), "selected production token {excluded}");
        }
        let production = strip_rust_test_modules(source);
        assert!(production.contains("mod fake"));
        assert!(production.contains("mod commented"));
        assert!(production.contains("production_between"));
        assert!(production.contains("production_after"));
        assert!(!production.contains("mod first"));
        assert!(!production.contains("mod second"));
    }
}
