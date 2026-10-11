use std::env;

#[must_use]
pub fn sanitize_snapshot_text(input: &str) -> String {
    let mut out = input.replace("\r\n", "\n");
    if let Ok(pwd) = env::current_dir() {
        out = replace_nonempty(out, &pwd.display().to_string(), "<ROOT>");
    }
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        out = replace_nonempty(out, &manifest_dir, "<ROOT>");
    }
    out = replace_temp_roots(
        out,
        env::var("TEST_TMP_DIR").ok().as_deref(),
        env::var("TMPDIR").ok().as_deref(),
        env::var("TMP").ok().as_deref(),
        env::var("TEMP").ok().as_deref(),
    );
    out = normalize_tmp_subdir(&out);
    if let Ok(home) = env::var("HOME") {
        out = replace_nonempty(out, &home, "<HOME>");
    }
    if let Ok(user) = env::var("USER") {
        out = replace_nonempty(out, &user, "<USER>");
    }
    if let Ok(logname) = env::var("LOGNAME") {
        out = replace_nonempty(out, &logname, "<USER>");
    }
    if let Ok(hostname) = env::var("HOSTNAME") {
        out = replace_nonempty(out, &hostname, "<HOSTNAME>");
    }
    if let Ok(hostname) = env::var("COMPUTERNAME") {
        out = replace_nonempty(out, &hostname, "<HOSTNAME>");
    }
    normalize_windows_path_separators(&out)
}

fn replace_temp_roots(
    mut input: String,
    test_tmp_dir: Option<&str>,
    tmpdir: Option<&str>,
    short_tmp_root: Option<&str>,
    platform_temp_root: Option<&str>,
) -> String {
    for root in [test_tmp_dir, tmpdir, short_tmp_root, platform_temp_root].into_iter().flatten() {
        input = replace_nonempty(input, root, "<TMPDIR>");
    }
    input
}

fn normalize_windows_path_separators(input: &str) -> String {
    if contains_windows_path(input) {
        input.replace('\\', "/")
    } else {
        input.to_string()
    }
}

fn contains_windows_path(input: &str) -> bool {
    let bytes = input.as_bytes();
    if bytes.starts_with(b"\\\\") {
        return true;
    }
    bytes.windows(3).enumerate().any(|(index, window)| {
        let has_drive_prefix =
            window[0].is_ascii_alphabetic() && window[1] == b':' && window[2] == b'\\';
        let has_boundary = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
        has_drive_prefix && has_boundary
    })
}

fn replace_nonempty(input: String, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        input
    } else {
        input.replace(needle, replacement)
    }
}

fn normalize_tmp_subdir(input: &str) -> String {
    let marker = "<TMPDIR>";
    let mut out = String::with_capacity(input.len());
    let mut idx = 0;
    while let Some(pos) = input[idx..].find(marker) {
        let start = idx + pos;
        out.push_str(&input[idx..start]);
        let after_marker = start + marker.len();
        let mut seg_start = after_marker;
        let bytes = input.as_bytes();
        if seg_start < bytes.len() && bytes[seg_start] == b'/' {
            seg_start += 1;
        }
        let mut seg_end = seg_start;
        while seg_end < bytes.len() {
            let byte = bytes[seg_end];
            if byte == b'/'
                || byte.is_ascii_whitespace()
                || byte == b','
                || byte == b')'
                || byte == b'"'
                || byte == b'\''
            {
                break;
            }
            seg_end += 1;
        }
        if seg_end > seg_start {
            out.push_str("<TMPDIR>/<TMP>");
        } else {
            out.push_str("<TMPDIR>");
        }
        idx = seg_end;
    }
    out.push_str(&input[idx..]);
    out
}

#[cfg(test)]
mod tests {
    use super::{normalize_tmp_subdir, replace_temp_roots};

    #[test]
    fn test_tmp_root_is_replaced_before_its_parent_scheduler_tmp_root() {
        let scheduler_tmp = "/local/slurmtmp.123";
        let test_tmp = "/local/slurmtmp.123/artifacts/ci/shards/one/tmp/test";
        let raw = format!("{test_tmp}/bam-plan-abc/out/result.json");

        let replaced = replace_temp_roots(raw, Some(test_tmp), Some(scheduler_tmp), None, None);

        assert_eq!(normalize_tmp_subdir(&replaced), "<TMPDIR>/<TMP>/out/result.json");
    }
}
