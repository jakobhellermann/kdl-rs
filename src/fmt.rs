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

pub(crate) fn autoformat_leading(leading: &mut String, config: &FormatConfig<'_>, is_first: bool) {
    let mut result = String::new();
    // Count leading newlines to detect blank lines before this item. For
    // a non-first item (subsequent node, or after another comment), a
    // single `\n` is "blank line before me" because the previous content
    // already terminated itself. For a first item (after opening brace or
    // document start), the first `\n` is just "I'm on a new line" and
    // doesn't represent a blank.
    let leading_newlines = leading.bytes().take_while(|&b| b == b'\n').count();
    let blank_lines_before = if is_first {
        leading_newlines.saturating_sub(1)
    } else {
        leading_newlines
    };
    // Cap at 1 to avoid runaway whitespace; one blank line is enough to
    // signal grouping.
    let blank_lines_before = blank_lines_before.min(1);
    for _ in 0..blank_lines_before {
        result.push('\n');
    }
    if !config.no_comments {
        let input = leading.trim();
        if !input.is_empty() {
            // Preserve blank lines between comment lines too.
            let mut prev_blank = false;
            let mut first = true;
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    if !first {
                        prev_blank = true;
                    }
                    continue;
                }
                if prev_blank {
                    result.push('\n');
                    prev_blank = false;
                }
                for _ in 0..config.indent_level {
                    result.push_str(config.indent);
                }
                writeln!(result, "{trimmed}").unwrap();
                first = false;
            }
        }
    }
    for _ in 0..config.indent_level {
        result.push_str(config.indent);
    }
    *leading = result;
}

pub(crate) fn autoformat_trailing(decor: &mut String, no_comments: bool) {
    if decor.is_empty() {
        return;
    }
    *decor = decor.trim().to_string();
    let mut result = String::new();
    if !decor.is_empty() && !no_comments {
        for comment in decor.lines() {
            writeln!(result, "{comment}").unwrap();
        }
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
