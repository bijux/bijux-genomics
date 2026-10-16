#![cfg(test)]

use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScratchAuthority {
    Other,
    CurrentDirectory,
    CurrentDirectoryArtifacts,
    Repository,
}

impl ScratchAuthority {
    fn merged(self, other: Self) -> Self {
        if self == Self::CurrentDirectoryArtifacts || other == Self::CurrentDirectoryArtifacts {
            Self::CurrentDirectoryArtifacts
        } else if self == Self::CurrentDirectory || other == Self::CurrentDirectory {
            Self::CurrentDirectory
        } else if self == Self::Repository || other == Self::Repository {
            Self::Repository
        } else {
            Self::Other
        }
    }
}

struct ScratchBinding<'a> {
    name: &'a str,
    authority: ScratchAuthority,
    scope: usize,
}

struct ScratchUpdate<'a> {
    at: usize,
    name: &'a str,
    authority: ScratchAuthority,
    scope: usize,
    declaration: bool,
    conditional: bool,
}

struct ScratchSource<'a> {
    tokens: Vec<crate::rust_source_tokens::SourceToken<'a>>,
    closing: Vec<Option<usize>>,
    bindings: Vec<ScratchBinding<'a>>,
    scopes: Vec<(usize, bool)>,
    updates: Vec<ScratchUpdate<'a>>,
}

impl<'a> ScratchSource<'a> {
    fn new(content: &'a str) -> Option<Self> {
        let tokens = crate::rust_source_tokens::source_tokens(content)?;
        let mut closing = vec![None; tokens.len()];
        let mut openings = Vec::new();
        for (index, token) in tokens.iter().enumerate().filter(|(_, token)| !token.literal) {
            match token.text {
                "(" | "[" | "{" => openings.push(index),
                ")" | "]" | "}" => {
                    let opening = openings.pop()?;
                    let expected = match token.text {
                        ")" => "(",
                        "]" => "[",
                        _ => "{",
                    };
                    if tokens[opening].text != expected {
                        return None;
                    }
                    closing[opening] = Some(index);
                }
                _ => {}
            }
        }
        if !openings.is_empty() {
            return None;
        }
        Some(Self {
            tokens,
            closing,
            bindings: Vec::new(),
            scopes: vec![(usize::MAX, false)],
            updates: Vec::new(),
        })
    }

    fn text(&self, index: usize) -> Option<&'a str> {
        self.tokens.get(index).map(|token| token.text)
    }

    fn binding(&self, name: &str) -> ScratchAuthority {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name == name)
            .map_or(ScratchAuthority::Other, |binding| binding.authority)
    }

    fn statement_end(&self, mut index: usize) -> Option<usize> {
        while index < self.tokens.len() {
            match self.text(index)? {
                ";" => return Some(index),
                "}" => return None,
                "(" | "[" | "{" => index = self.closing[index]? + 1,
                _ => index += 1,
            }
        }
        None
    }

    fn path_argument(&self, start: usize, end: usize) -> Option<&'a str> {
        let token = self.tokens.get(start).filter(|_| start < end)?;
        if token.literal {
            let text = token.text;
            let quoted = if text.starts_with('"') && text.ends_with('"') {
                Some(&text[1..text.len() - 1])
            } else if text.starts_with('r') {
                text.find('"')
                    .zip(text.rfind('"'))
                    .filter(|(first, last)| first < last)
                    .map(|(first, last)| &text[first + 1..last])
            } else {
                None
            };
            return quoted;
        }
        for prefix in [["Path", "::", "new", "("], ["PathBuf", "::", "from", "("]] {
            if self.tokens.get(start..start + 4).is_some_and(|tokens| {
                tokens
                    .iter()
                    .zip(prefix)
                    .all(|(token, expected)| !token.literal && token.text == expected)
            }) {
                return self.path_argument(start + 4, end);
            }
        }
        None
    }

    // Only recognized filesystem path arguments establish an effect boundary.
    // Bare imported functions lose that authority when a local callable shadows them.
    fn filesystem_path_call(&self, path: &[&str]) -> bool {
        let unbound =
            path.len() == 1 && !self.bindings.iter().any(|binding| binding.name == path[0]);
        (unbound && matches!(path[0], "write" | "create_dir" | "create_dir_all"))
            || matches!(
                path,
                ["fs", "write" | "create_dir" | "create_dir_all"]
                    | ["std", "fs", "write" | "create_dir" | "create_dir_all"]
                    | ["File", "create" | "open"]
                    | ["fs", "File", "create" | "open"]
                    | ["std", "fs", "File", "create" | "open"]
            )
    }

    fn first_argument_end(&self, mut cursor: usize, end: usize) -> usize {
        while cursor < end {
            match self.text(cursor) {
                Some(",") => break,
                Some("(" | "[" | "{") => {
                    let Some(close) = self.closing[cursor] else { return end };
                    cursor = close + 1;
                }
                _ => cursor += 1,
            }
        }
        cursor
    }

    fn chain(&self, start: usize, end: usize) -> (ScratchAuthority, usize, bool) {
        let mut cursor = start;
        let mut authority = ScratchAuthority::Other;
        let mut forbidden = false;
        if self.text(cursor) == Some("(") {
            if let Some(close) = self.closing[cursor].filter(|&close| close < end) {
                let result = self.expression(cursor + 1, close);
                authority = result.0;
                forbidden |= result.1;
                cursor = close + 1;
            }
        } else if self.tokens.get(cursor).is_some_and(|token| !token.literal) {
            let mut path = vec![self.tokens[cursor].text];
            cursor += 1;
            while cursor + 1 < end && self.text(cursor) == Some("::") {
                path.push(self.tokens[cursor + 1].text);
                cursor += 2;
            }
            if self.text(cursor) == Some("(") {
                if let Some(close) = self.closing[cursor].filter(|&close| close < end) {
                    let unbound_current_dir = path.as_slice() == ["current_dir"]
                        && !self.bindings.iter().any(|binding| binding.name == "current_dir");
                    if close == cursor + 1
                        && (unbound_current_dir
                            || matches!(
                                path.as_slice(),
                                ["env", "current_dir"] | ["std", "env", "current_dir"]
                            ))
                    {
                        authority = ScratchAuthority::CurrentDirectory;
                    } else if close == cursor + 1
                        && path.as_slice() == ["workspace_paths", "repo_root"]
                    {
                        authority = ScratchAuthority::Repository;
                    }
                    if self.filesystem_path_call(&path) {
                        let argument_end = self.first_argument_end(cursor + 1, close);
                        let argument = self.expression(cursor + 1, argument_end);
                        forbidden |=
                            argument.0 == ScratchAuthority::CurrentDirectoryArtifacts || argument.1;
                    }
                    cursor = close + 1;
                }
            } else if path.len() == 1 {
                authority = self.binding(path[0]);
            }
        } else {
            cursor += 1;
        }
        while cursor < end {
            if self.text(cursor) == Some("?") {
                cursor += 1;
                continue;
            }
            if self.text(cursor) != Some(".") {
                break;
            }
            let Some(method) = self.text(cursor + 1) else { break };
            if self.text(cursor + 2) != Some("(") {
                authority = ScratchAuthority::Other;
                cursor += 2;
                continue;
            }
            let Some(close) = self.closing[cursor + 2].filter(|&close| close < end) else { break };
            if method == "join" {
                if authority == ScratchAuthority::CurrentDirectory
                    && self
                        .path_argument(cursor + 3, close)
                        .is_some_and(|path| path == "artifacts" || path.starts_with("artifacts/"))
                {
                    authority = ScratchAuthority::CurrentDirectoryArtifacts;
                }
                let argument = self.expression(cursor + 3, close);
                forbidden |= argument.1;
                if argument.0 == ScratchAuthority::Repository {
                    authority = ScratchAuthority::Repository;
                } else if self
                    .path_argument(cursor + 3, close)
                    .is_some_and(|path| Path::new(path).is_absolute())
                {
                    authority = ScratchAuthority::Other;
                }
            } else if matches!(
                method,
                "is_absolute"
                    | "is_relative"
                    | "is_file"
                    | "is_dir"
                    | "exists"
                    | "starts_with"
                    | "ends_with"
            ) {
                authority = ScratchAuthority::Other;
            }
            cursor = close + 1;
        }
        (authority, cursor.max(start + 1), forbidden)
    }

    fn expression(&self, mut index: usize, end: usize) -> (ScratchAuthority, bool) {
        let mut authority = ScratchAuthority::Other;
        let mut forbidden = false;
        while index < end {
            if !self.tokens[index].literal
                && !matches!(self.text(index.saturating_sub(1)), Some("." | "::"))
            {
                let result = self.chain(index, end);
                authority = authority.merged(result.0);
                forbidden |= result.2;
                index = result.1;
            } else {
                index += 1;
            }
        }
        (authority, forbidden)
    }

    fn apply_updates(&mut self, index: usize) {
        for update in std::mem::take(&mut self.updates) {
            if update.at != index {
                self.updates.push(update);
                continue;
            }
            if update.declaration {
                self.bindings.push(ScratchBinding {
                    name: update.name,
                    authority: update.authority,
                    scope: update.scope,
                });
            } else if let Some(binding) =
                self.bindings.iter_mut().rev().find(|binding| binding.name == update.name)
            {
                binding.authority = if update.conditional {
                    binding.authority.merged(update.authority)
                } else {
                    update.authority
                };
            }
        }
    }

    fn schedule(&mut self, name: usize, equals: usize, declaration: bool) {
        let Some(end) = self.statement_end(equals + 1) else { return };
        let value = self.expression(equals + 1, end).0;
        let scope = self.scopes.last().expect("scratch source root scope").0;
        self.updates.push(ScratchUpdate {
            at: end,
            name: self.tokens[name].text,
            authority: value,
            scope,
            declaration,
            conditional: !declaration && self.scopes.iter().any(|(_, conditional)| *conditional),
        });
    }

    fn derives_scratch(mut self) -> bool {
        let mut declarations = Vec::new();
        for index in 0..self.tokens.len() {
            self.apply_updates(index);
            if self.tokens[index].literal {
                continue;
            }
            let text = self.tokens[index].text;
            if text == "{" {
                let from = (0..index)
                    .rev()
                    .find(|&at| matches!(self.text(at), Some(";" | "{" | "}")))
                    .map_or(0, |at| at + 1);
                let conditional = self.tokens[from..index].iter().any(|token| {
                    !token.literal
                        && matches!(token.text, "if" | "else" | "match" | "while" | "for" | "loop")
                });
                self.scopes.push((index, conditional));
            } else if text == "}" {
                if let Some((scope, _)) = self.scopes.pop() {
                    self.bindings.retain(|binding| binding.scope != scope);
                }
            } else if text == "let" {
                let mut name = index + 1;
                while matches!(self.text(name), Some("mut" | "ref")) {
                    name += 1;
                }
                if let Some(end) = self.statement_end(name + 1) {
                    if let Some(equals) = (name + 1..end)
                        .find(|&at| self.text(at) == Some("=") && self.text(at + 1) != Some("="))
                    {
                        declarations.push(name);
                        self.schedule(name, equals, true);
                    }
                }
            } else if self.text(index + 1) == Some("=")
                && self.text(index + 2) != Some("=")
                && !declarations.contains(&index)
                && !matches!(self.text(index.saturating_sub(1)), Some("." | "::"))
            {
                self.schedule(index, index + 1, false);
            }
            if !matches!(self.text(index.saturating_sub(1)), Some("." | "::"))
                && self.chain(index, self.tokens.len()).2
            {
                return true;
            }
        }
        false
    }
}

pub(super) fn derives_scratch_from_current_dir(content: &str) -> bool {
    ScratchSource::new(content).is_none_or(ScratchSource::derives_scratch)
}
