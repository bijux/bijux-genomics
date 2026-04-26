pub(super) fn build_command_string(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        return shell_token(command);
    }
    std::iter::once(command)
        .chain(args.iter().map(String::as_str))
        .map(shell_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_token(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(ch, '.' | '_' | '/' | '-' | ':' | '=' | '@' | '+' | ',' | '%')
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::build_command_string;

    #[test]
    fn command_string_quotes_args_with_spaces() {
        let args = vec![
            "--input".to_string(),
            "/workspace/sample reads.fq".to_string(),
            "O'Reilly".to_string(),
        ];

        assert_eq!(
            build_command_string("bijux-tool", &args),
            "bijux-tool --input '/workspace/sample reads.fq' 'O'\"'\"'Reilly'"
        );
    }

    #[test]
    fn command_string_quotes_empty_command() {
        assert_eq!(build_command_string("", &[]), "''");
    }
}
