//! Lexical command diagnostics; this is not a shell interpreter or execution proof.
pub(super) fn is_shell_wrapper(executable: &str) -> bool {
    matches!(basename_lower(executable).as_deref(), Some("sh" | "bash"))
}

pub(super) fn shell_payload(argv: &[String]) -> &str {
    argv.last().map_or("", std::string::String::as_str)
}

pub(super) fn direct_command_has_real_invocation(executable: &str) -> bool {
    let Some(base) = basename_lower(executable) else {
        return false;
    };
    !matches!(base.as_str(), "" | "echo" | "true" | ":" | "cd" | "mkdir" | "export" | "set")
}

pub(super) fn shell_command_has_real_invocation(command_heads: &[String]) -> bool {
    command_heads.iter().any(|head| {
        !matches!(head.as_str(), "" | "echo" | "true" | ":" | "cd" | "mkdir" | "export" | "set")
    })
}

pub(super) fn is_echo_only_execution(
    executable: &str,
    payload: &str,
    command_heads: &[String],
) -> bool {
    if matches!(basename_lower(executable).as_deref(), Some("echo")) {
        return true;
    }
    if !is_shell_wrapper(executable) {
        return false;
    }
    if command_heads.is_empty() {
        let trimmed = payload.trim().to_ascii_lowercase();
        return trimmed.starts_with("echo ");
    }
    command_heads.iter().all(|head| head == "echo")
}

pub(super) fn is_unconditional_success(
    executable: &str,
    payload: &str,
    command_heads: &[String],
    lowered_command: &str,
) -> bool {
    if matches!(basename_lower(executable).as_deref(), Some("true")) {
        return true;
    }
    if lowered_command.trim() == ":" {
        return true;
    }
    if !is_shell_wrapper(executable) {
        return false;
    }
    if command_heads.is_empty() {
        return payload_is_standalone_success(payload);
    }
    payload_is_standalone_success(payload)
}

fn payload_is_standalone_success(payload: &str) -> bool {
    let normalized_lines = payload
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    match normalized_lines.as_slice() {
        [line] => matches!(line.as_str(), ":" | "true" | "exit 0" | "rc=0"),
        [assign, exit] => {
            assign == "rc=0" && matches!(exit.as_str(), "exit 0" | "exit $rc" | "exit \"$rc\"")
        }
        _ => false,
    }
}

pub(super) fn extract_shell_command_heads(payload: &str) -> Vec<String> {
    let mut heads = Vec::new();
    let mut token = String::new();
    let mut current_head: Option<String> = None;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape_next = false;

    let flush_token = |token: &mut String, current_head: &mut Option<String>| {
        if token.is_empty() {
            return;
        }
        let value = std::mem::take(token);
        if current_head.is_none() && !is_shell_control_token(&value) && !is_shell_assignment(&value)
        {
            *current_head = Some(value);
        }
    };
    let flush_segment = |heads: &mut Vec<String>, current_head: &mut Option<String>| {
        if let Some(head) = current_head.take() {
            heads.push(normalize_command_head(&head));
        }
    };

    for ch in payload.chars() {
        if escape_next {
            token.push(ch);
            escape_next = false;
            continue;
        }
        if ch == '\\' && !in_single {
            escape_next = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if !in_single && !in_double {
            if matches!(ch, ' ' | '\t' | '\r') {
                flush_token(&mut token, &mut current_head);
                continue;
            }
            if matches!(ch, '\n' | ';' | '|' | '&') {
                flush_token(&mut token, &mut current_head);
                flush_segment(&mut heads, &mut current_head);
                continue;
            }
        }
        token.push(ch);
    }
    flush_token(&mut token, &mut current_head);
    flush_segment(&mut heads, &mut current_head);
    heads
}

fn is_shell_control_token(token: &str) -> bool {
    let token = token.trim();
    token.is_empty()
        || matches!(
            token,
            "if" | "then"
                | "else"
                | "fi"
                | "do"
                | "done"
                | "for"
                | "while"
                | "case"
                | "esac"
                | "in"
                | "["
                | "]"
                | "test"
                | "{"
                | "}"
                | "function"
        )
        || token.ends_with("()")
        || token.ends_with("(){")
}

fn is_shell_assignment(token: &str) -> bool {
    let Some((name, _value)) = token.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn normalize_command_head(head: &str) -> String {
    basename_lower(head).unwrap_or_else(|| head.trim().to_ascii_lowercase())
}

fn basename_lower(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let candidate = trimmed.rsplit('/').next().unwrap_or(trimmed);
    Some(candidate.to_ascii_lowercase())
}
