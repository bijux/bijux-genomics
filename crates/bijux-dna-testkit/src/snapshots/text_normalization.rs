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
    if let Ok(tmpdir) = env::var("TMPDIR") {
        out = replace_nonempty(out, &tmpdir, "<TMPDIR>");
    }
    if let Ok(tmp) = env::var("TMP") {
        out = replace_nonempty(out, &tmp, "<TMPDIR>");
    }
    if let Ok(temp) = env::var("TEMP") {
        out = replace_nonempty(out, &temp, "<TMPDIR>");
    }
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
    out.replace('\\', "/")
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
