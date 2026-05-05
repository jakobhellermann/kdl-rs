use std::fmt::Write as _;

/// Formatting configuration for use with [`KdlDocument::autoformat_config`](`crate::KdlDocument::autoformat_config`)
/// and [`KdlNode::autoformat_config`](`crate::KdlNode::autoformat_config`).
#[non_exhaustive]
#[derive(Debug)]
pub struct FormatConfig<'a> {
    /// How deeply to indent the overall node or document,
    /// in repetitions of [`indent`](`FormatConfig::indent`).
    /// Defaults to `0`.
    pub indent_level: usize,

    /// The indentation to use at each level. Defaults to four spaces.
    pub indent: &'a str,

    /// Whether to remove comments. Defaults to `false`.
    pub no_comments: bool,

    /// Whether to keep individual entry formatting.
    pub entry_autoformate_keep: bool,
}

/// See field documentation for defaults.
impl Default for FormatConfig<'_> {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl FormatConfig<'_> {
    /// Creates a new [`FormatConfigBuilder`] with default configuration.
    pub const fn builder() -> FormatConfigBuilder<'static> {
        FormatConfigBuilder::new()
    }
}

/// A [`FormatConfig`] builder.
///
/// Note that setters can be repeated.
#[derive(Debug, Default)]
pub struct FormatConfigBuilder<'a>(FormatConfig<'a>);

impl<'a> FormatConfigBuilder<'a> {
    /// Creates a new [`FormatConfig`] builder with default configuration.
    pub const fn new() -> Self {
        Self(FormatConfig {
            indent_level: 0,
            indent: "    ",
            no_comments: false,
            entry_autoformate_keep: false,
        })
    }

    /// How deeply to indent the overall node or document,
    /// in repetitions of [`indent`](`FormatConfig::indent`).
    /// Defaults to `0` iff not specified.
    pub const fn maybe_indent_level(mut self, indent_level: Option<usize>) -> Self {
        if let Some(indent_level) = indent_level {
            self.0.indent_level = indent_level;
        }
        self
    }

    /// How deeply to indent the overall node or document,
    /// in repetitions of [`indent`](`FormatConfig::indent`).
    /// Defaults to `0` iff not specified.
    pub const fn indent_level(mut self, indent_level: usize) -> Self {
        self.0.indent_level = indent_level;
        self
    }

    /// The indentation to use at each level.
    /// Defaults to four spaces iff not specified.
    pub const fn maybe_indent<'b, 'c>(self, indent: Option<&'b str>) -> FormatConfigBuilder<'c>
    where
        'a: 'b,
        'b: 'c,
    {
        if let Some(indent) = indent {
            self.indent(indent)
        } else {
            self
        }
    }

    /// The indentation to use at each level.
    /// Defaults to four spaces if not specified.
    pub const fn indent(self, indent: &str) -> FormatConfigBuilder<'_> {
        FormatConfigBuilder(FormatConfig { indent, ..self.0 })
    }

    /// Whether to remove comments.
    /// Defaults to `false` iff not specified.
    pub const fn maybe_no_comments(mut self, no_comments: Option<bool>) -> Self {
        if let Some(no_comments) = no_comments {
            self.0.no_comments = no_comments;
        }
        self
    }

    /// Whether to remove comments.
    /// Defaults to `false` iff not specified.
    pub const fn no_comments(mut self, no_comments: bool) -> Self {
        self.0.no_comments = no_comments;
        self
    }

    /// Builds the [`FormatConfig`].
    pub const fn build(self) -> FormatConfig<'a> {
        self.0
    }
}

/// Returns true when every non-blank line in `s` starts with `//`. Used to
/// decide whether decor is safe to re-indent: if the block contains any
/// non-comment content (e.g. a slashdashed node with children), we have to
/// preserve it verbatim because we don't know how to format it.
fn all_lines_are_comments(s: &str) -> bool {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .all(|l| l.starts_with("//"))
}

fn write_indent(out: &mut String, config: &FormatConfig<'_>) {
    for _ in 0..config.indent_level {
        out.push_str(config.indent);
    }
}

/// Emit `content` (already trimmed) as a sequence of comment lines,
/// re-indented at `indent`'s level. Blank lines between comments are
/// preserved as bare `\n`. Caller guarantees `all_lines_are_comments`.
fn write_comment_block(out: &mut String, content: &str, indent: Option<&FormatConfig<'_>>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        if let Some(cfg) = indent {
            write_indent(out, cfg);
        }
        writeln!(out, "{trimmed}").unwrap();
    }
}

/// Emit decor content, either as a re-indented comment block (if every
/// non-blank line is a `//` comment) or verbatim (slashdashed nodes etc.,
/// which we don't know how to re-indent). Always ends with `\n` when
/// non-empty.
fn write_decor_content(out: &mut String, content: &str, indent: Option<&FormatConfig<'_>>) {
    if all_lines_are_comments(content) {
        write_comment_block(out, content, indent);
    } else {
        out.push_str(content);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
}

pub(crate) fn autoformat_leading(leading: &mut String, config: &FormatConfig<'_>, is_first: bool) {
    // Count leading newlines to detect a blank line before this item. For
    // a non-first item, a single `\n` is "blank line before me" because the
    // previous content already terminated itself. For a first item (after
    // opening brace or document start), the first `\n` is just "I'm on a
    // new line" and doesn't represent a blank. Cap at 1 — one blank line is
    // enough to signal grouping.
    let leading_newlines = leading.bytes().take_while(|&b| b == b'\n').count();
    let blank_before = if is_first {
        leading_newlines > 1
    } else {
        leading_newlines > 0
    };
    // Detect a blank line *between* the last comment line and the node
    // itself: two-or-more trailing newlines (one to terminate the last
    // comment, plus one per blank line).
    let trailing_blank_after_comments = leading
        .bytes()
        .rev()
        .take_while(|&b| matches!(b, b'\n' | b' ' | b'\t'))
        .filter(|&b| b == b'\n')
        .count()
        > 1;

    let mut result = String::new();
    if blank_before {
        result.push('\n');
    }
    if !config.no_comments {
        let content = leading.trim();
        if !content.is_empty() {
            write_decor_content(&mut result, content, Some(config));
            if trailing_blank_after_comments {
                result.push('\n');
            }
        }
    }
    write_indent(&mut result, config);
    *leading = result;
}

pub(crate) fn autoformat_trailing(decor: &mut String, no_comments: bool) {
    autoformat_trailing_indented(decor, no_comments, None, true);
}

/// Like [`autoformat_trailing`], but re-indents each non-empty line at the
/// given config's indent level. Used for trailing decor that lives at the
/// statement level (between nodes, at the end of a children block).
///
/// `preceded_by_content` is true when something appeared before this
/// trailing decor in the same container (a previous node, or the outer
/// document). When false (e.g. a children block with no nodes, where the
/// trailing decor is the only content), a single leading newline is just
/// "this content starts on a new line after the opening brace" rather
/// than a blank line, mirroring the same distinction in
/// [`autoformat_leading`].
pub(crate) fn autoformat_trailing_indented(
    decor: &mut String,
    no_comments: bool,
    indent: Option<&FormatConfig<'_>>,
    preceded_by_content: bool,
) {
    if decor.is_empty() || no_comments {
        if no_comments {
            decor.clear();
        }
        return;
    }
    let leading_newlines = decor
        .bytes()
        .take_while(|&b| matches!(b, b'\n' | b' ' | b'\t'))
        .filter(|&b| b == b'\n')
        .count();
    let leading_blank = if preceded_by_content {
        leading_newlines > 0
    } else {
        leading_newlines > 1
    };
    let content = decor.trim();
    let mut result = String::new();
    if !content.is_empty() {
        if leading_blank {
            result.push('\n');
        }
        write_decor_content(&mut result, content, indent);
    }
    *decor = result;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builder() -> miette::Result<()> {
        let built = FormatConfig::builder()
            .indent_level(12)
            .indent(" \t")
            .no_comments(true)
            .build();
        assert!(matches!(
            built,
            FormatConfig {
                indent_level: 12,
                indent: " \t",
                no_comments: true,
                entry_autoformate_keep: false,
            }
        ));
        Ok(())
    }
}
