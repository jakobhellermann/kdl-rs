use insta::assert_snapshot;
use kdl::{KdlDocument, KdlNode};

#[test]
fn build_and_format() {
    let mut c = KdlNode::new("c");
    c.ensure_children();
    let mut b = KdlNode::new("b");
    b.ensure_children().nodes_mut().push(c);
    let mut a = KdlNode::new("a");
    a.ensure_children().nodes_mut().push(b);

    let mut doc = KdlDocument::new();
    doc.nodes_mut().push(a);
    doc.autoformat();
    let fmt = doc.to_string();
    // `c` has empty children, which renders inline. The outer blocks are
    // multi-line because they have multiple/non-inline content.
    assert_eq!(
        fmt,
        r#"a {
    b {
        c { }
    }
}
"#
    );
}

#[test]
#[cfg(feature = "v1")]
fn ensure_v1_does_not_over_escape_forward_slash() {
    // Per the v1 spec, `\/` is a permitted escape but unescaped `/` is also
    // legal. ensure_v1 should not introduce unnecessary `\/` escapes.
    let input = "node \"a/b/c\"\n";
    let mut doc = KdlDocument::parse_v1(input).unwrap();
    doc.ensure_v1();
    assert_eq!(doc.to_string(), input);
}

#[test]
#[cfg(feature = "v1")]
fn ensure_v2_strips_escaped_forward_slash() {
    // `\/` is forbidden in v2, so ensure_v2 must convert it to a `/`.
    let input = "node \"a\\/b\"\n";
    let mut doc = KdlDocument::parse_v1(input).unwrap();
    doc.ensure_v2();
    assert_eq!(doc.to_string(), "node \"a/b\"\n");
}

#[test]
#[cfg(feature = "v1")]
fn ensure_v1_preserves_raw_string_with_backslash_slash() {
    // In a raw string, `\/` is two literal characters, and should be kept as is.
    let input = "node r#\"a\\/b\"#\n";
    let mut doc = KdlDocument::parse_v1(input).unwrap();
    doc.ensure_v1();
    assert_eq!(doc.to_string(), input);
}

#[track_caller]
fn format_no_comments(input: &str) -> String {
    let mut doc = KdlDocument::parse_v2(input).unwrap();
    doc.autoformat_no_comments();
    doc.to_string()
}

#[track_caller]
fn format(input: &str) -> String {
    let mut doc = match KdlDocument::parse_v2(input) {
        Ok(doc) => doc,
        Err(e) => {
            let mut rendered = String::new();
            miette::GraphicalReportHandler::new()
                .with_theme(miette::GraphicalTheme::unicode_nocolor())
                .render_report(&mut rendered, &e)
                .unwrap();
            panic!("failed to parse KDL document:\n{rendered}");
        }
    };
    doc.autoformat();
    doc.to_string()
}

#[cfg(feature = "v1")]
#[track_caller]
fn format_v1(input: &str) -> String {
    let mut doc = match KdlDocument::parse_v1(input) {
        Ok(doc) => doc,
        Err(e) => {
            let mut rendered = String::new();
            miette::GraphicalReportHandler::new()
                .with_theme(miette::GraphicalTheme::unicode_nocolor())
                .render_report(&mut rendered, &e)
                .unwrap();
            panic!("failed to parse KDL v1 document:\n{rendered}");
        }
    };
    doc.autoformat();
    doc.to_string()
}

#[test]
fn format_example() {
    let input = r#"
// https://yalter.github.io/niri/Configuration:-Introduction

include "input.kdl"
include "workspaces.kdl"

screenshot-path "~/Pictures/Screenshots/%Y-%m-%d %H:%M:%S.png"

spawn-at-startup "awww-daemon"
spawn-at-startup "awww-daemon" "-n" "backdrop"
spawn-at-startup "swayosd-server"
spawn-at-startup "~/.local/share/scripts/launch-waybar"
spawn-sh-at-startup "~/.local/share/scripts/niri_tile_to_n -n3 -x false -xc false"

// TODO
// - screenshot window
// - toggle column for next window
"#;

    assert_snapshot!(format(input), @r#"
    // https://yalter.github.io/niri/Configuration:-Introduction

    include input.kdl
    include workspaces.kdl

    screenshot-path "~/Pictures/Screenshots/%Y-%m-%d %H:%M:%S.png"

    spawn-at-startup awww-daemon
    spawn-at-startup awww-daemon -n backdrop
    spawn-at-startup swayosd-server
    spawn-at-startup "~/.local/share/scripts/launch-waybar"
    spawn-sh-at-startup "~/.local/share/scripts/niri_tile_to_n -n3 -x false -xc false"

    // TODO
    // - screenshot window
    // - toggle column for next window
    "#);
}

#[test]
fn format_comment_newline() {
    let input = r#"
option "yes"
// TODO
"#;

    assert_snapshot!(format(input), @"
    option yes
    // TODO
    ");
}

#[test]
fn format_comment_end_of_line() {
    let input = r#"
option "yes" // foo
"#;

    assert_snapshot!(format(input), @"option yes // foo");
}

// Same input as `format_comment_end_of_line`, but parsed as v1. The v1
// grammar doesn't include single-line-comment as a node terminator, so the
// `// foo` lands in the node's `trailing` field with a leading space.
#[test]
#[cfg(feature = "v1")]
fn format_v1_comment_end_of_line() {
    let input = "option \"yes\" // foo\n";
    assert_snapshot!(format_v1(input), @"option yes // foo");
}

#[test]
fn format_comment_newline_indented() {
    let input = r#"
input {
    natural-scroll
    // accel-speed 0.2
    // accel-profile \"flat\"
}
"#;

    assert_snapshot!(format(input), @r#"
    input {
        natural-scroll
        // accel-speed 0.2
        // accel-profile \"flat\"
    }
    "#);
}

#[test]
fn format_comment_only_inside_children() {
    let input = r#"
input {
    // just a comment, no nodes
}
"#;
    assert_snapshot!(format(input), @"
    input {
        // just a comment, no nodes
    }
    ");
}

#[test]
fn format_blank_lines_between_top_level_nodes() {
    let input = r#"
node "a"

node "b"
"#;
    assert_snapshot!(format(input), @"
    node a

    node b
    ");
}

#[test]
fn format_blank_lines_between_node_comment() {
    let input = r#"
// a

node a

// b

node b

// c
"#;
    assert_snapshot!(format(input), @"
    // a

    node a

    // b

    node b

    // c
    ");
}

#[test]
fn format_blank_lines_between_top_level_comments() {
    let input = r#"
// first

// second
"#;
    assert_snapshot!(format(input), @"
    // first

    // second
    ");
}

#[test]
fn format_semicolon_terminator() {
    let input = r#"node "a"; node "b""#;
    assert_snapshot!(format(input), @r#"
    node a
    node b
    "#);
}

#[test]
fn format_slashdash_node() {
    let input = r#"
/-commented "out"
node "kept"
"#;
    assert_snapshot!(format(input), @r#"
    /-commented "out"
    node kept
    "#);
}

// BUG: slashdashed entries are silently dropped during autoformat. The
// `/-key="hidden"` should round-trip; instead it disappears.
#[test]
fn format_slashdash_entry() {
    let input = r#"node "a" /-key="hidden" "b""#;
    assert_snapshot!(format(input), @"node a b");
}

#[test]
fn format_end_of_line_comment_on_node_with_children() {
    let input = r#"
node "a" { // header
    inner
}
"#;
    assert_snapshot!(format(input), @"
    node a {
        // header
        inner
    }
    ");
}

#[test]
fn format_no_comments_strips_inline_comment() {
    let input = "option \"yes\" // foo\n";
    let mut doc = KdlDocument::parse(input).unwrap();
    doc.autoformat_no_comments();
    assert_snapshot!(doc.to_string(), @"option yes");
}

#[test]
fn format_no_comments_strips_standalone_comment() {
    let input = r#"
// header
node "a"
"#;
    let mut doc = KdlDocument::parse(input).unwrap();
    doc.autoformat_no_comments();
    assert_snapshot!(doc.to_string(), @"node a");
}

// Blank lines that exist purely to space out comments collapse along with
// the comments themselves.
#[test]
fn format_no_comments_collapses_comment_only_blanks() {
    let input = r#"
// a

// b
node "x"
"#;
    assert_snapshot!(format_no_comments(input), @"node x");
}

// Blank lines between actual nodes are preserved even when comments are
// stripped.
#[test]
fn format_no_comments_keeps_blank_lines_between_nodes() {
    let input = r#"
node "a"

// in between
node "b"
"#;
    assert_snapshot!(format_no_comments(input), @"
    node a

    node b
    ");
}

// A children block that contained only comments collapses to an empty inline
// `{ }` once the comments are gone.
#[test]
fn format_no_comments_empties_comment_only_children() {
    let input = r#"
input {
    // just a comment, no nodes
}
"#;
    assert_snapshot!(format_no_comments(input), @"input { }");
}

// The end-of-line comment after `{` is dropped; the inner node renders
// normally.
#[test]
fn format_no_comments_strips_end_of_line_comment_on_children() {
    let input = r#"
node "a" { // header
    inner
}
"#;
    assert_snapshot!(format_no_comments(input), @"
    node a {
        inner
    }
    ");
}

// A slashdashed node uses `/-` (not `//`), so semantically it is not a
// comment — but the parser puts it in leading decor, which `no_comments`
// drops wholesale. Pinned to surface the behavior; arguably a bug.
#[test]
fn format_no_comments_drops_slashdash_node() {
    let input = r#"
/-commented "out"
node "kept"
"#;
    assert_snapshot!(format_no_comments(input), @"node kept");
}

// Same wholesale-drop applies to multi-line slashdashed blocks.
#[test]
fn format_no_comments_drops_slashdash_block_too() {
    let input = r#"
// header
/-window-rule {
    geometry-corner-radius 8
}
// trailer
node
"#;
    assert_snapshot!(format_no_comments(input), @"node");
}

// Trailing comment on the document (after the last node) lives in
// `doc.trailing` and must also be stripped under `no_comments`.
#[test]
fn format_no_comments_strips_doc_trailing_comment() {
    let input = "node\n// trailer\n";
    assert_snapshot!(format_no_comments(input), @"node");
}

// Same shape with a blank line before the trailing comment — still
// stripped, no orphan blank line left behind.
#[test]
fn format_no_comments_strips_doc_trailing_comment_with_blank() {
    let input = "node\n\n// trailer\n";
    assert_snapshot!(format_no_comments(input), @"node");
}

// Inline comment that the v1 parser stuffs into `trailing` (rather than the
// terminator) is also stripped.
#[test]
#[cfg(feature = "v1")]
fn format_v1_no_comments_strips_inline_comment() {
    let input = "option \"yes\" // foo\n";
    let mut doc = KdlDocument::parse_v1(input).unwrap();
    doc.autoformat_no_comments();
    assert_snapshot!(doc.to_string(), @"option yes");
}

#[test]
fn format_custom_indent_two_spaces() {
    let input = r#"
outer {
    inner {
        deep
    }
}
"#;
    let mut doc = KdlDocument::parse(input).unwrap();
    doc.autoformat_config(&kdl::FormatConfig::builder().indent("  ").build());
    assert_snapshot!(doc.to_string(), @"
    outer {
      inner {
        deep
      }
    }
    ");
}

#[test]
fn format_custom_indent_tab() {
    let input = r#"
outer {
    inner
}
"#;
    let mut doc = KdlDocument::parse(input).unwrap();
    doc.autoformat_config(&kdl::FormatConfig::builder().indent("\t").build());
    assert_snapshot!(doc.to_string(), @"
    outer {
    	inner
    }
    ");
}

#[test]
fn format_remove_unnecessary_space() {
    let input = r#"
binds  {
    Mod+o   hotkey-overlay-title  =  "null"  {  show-hotkey-overlay;  }
}
"#;

    assert_snapshot!(format(input), @r#"
    binds {
        Mod+o hotkey-overlay-title="null" { show-hotkey-overlay; }
    }
    "#);
}

// A multi-line slashdashed node-with-children round-trips with its inner
// indentation intact.
#[test]
fn format_slashdash_block() {
    let input = r#"
/-window-rule {
    geometry-corner-radius 8
    clip-to-geometry "true"
}
"#;

    assert_snapshot!(format(input), @r#"
    /-window-rule {
        geometry-corner-radius 8
        clip-to-geometry "true"
    }
    "#);
}

// Same input as `format_slashdash_block`, parsed as v1. v1 parses the
// slashdashed block into the document's trailing rather than leading, so
// the same preservation rule applies on the trailing-decor path.
#[test]
#[cfg(feature = "v1")]
fn format_v1_slashdash_block() {
    let input = r#"
/-window-rule {
    geometry-corner-radius 8
    clip-to-geometry "true"
}
"#;

    assert_snapshot!(format_v1(input), @r#"
    /-window-rule {
        geometry-corner-radius 8
        clip-to-geometry "true"
    }
    "#);
}

// Children block whose first item is a comment, with no blank line in the
// source. The comment should sit directly under the opening brace.
#[test]
fn format_children_starting_with_comment() {
    let input = r#"
trackpoint {
    // off
    // natural-scroll
}
"#;

    assert_snapshot!(format(input), @r#"
    trackpoint {
        // off
        // natural-scroll
    }
    "#);
}

// Multiple consecutive blank lines collapse to a single blank. Documents
// the cap; surfaced in real configs (kdlfmt diff against niri's ui.kdl).
#[test]
fn format_collapses_multiple_blank_lines() {
    let input = r#"
node "a"



node "b"
"#;
    assert_snapshot!(format(input), @"
    node a

    node b
    ");
}

// `a { b }` (single-line block, one child) round-trips as single-line.
#[test]
fn format_singleline_block_one_child() {
    let input = "a { b }\n";
    assert_snapshot!(format(input), @"a { b }");
}

// `a { b\n}` (trailing newline before `}` only) collapses to single-line:
// the inner newline is just trailing whitespace.
#[test]
fn format_singleline_block_trailing_newline_collapses() {
    let input = "a { b\n}\n";
    assert_snapshot!(format(input), @"a { b }");
}

// `a { b; }` (semicolon-terminated single child): the `;` is preserved
// since the source had one. Stripping it would (a) silently drop a
// deliberately-written separator and (b) produce invalid v1.
#[test]
fn format_singleline_block_with_semicolon() {
    let input = "a { b; }\n";
    assert_snapshot!(format(input), @"a { b; }");
}

// `a { b; }` parsed as v1: `;` is preserved by autoformat, since the source
// had it. (v1 grammar requires `;` here — `}` is not an implicit
// terminator — so stripping it would produce invalid v1.)
#[test]
#[cfg(feature = "v1")]
fn format_v1_singleline_block_keeps_semicolon() {
    let input = "a { b; }\n";
    assert_snapshot!(format_v1(input), @"a { b; }");
}

// `a { b }` parsed as v2 (where it's legal), then run through `ensure_v1`:
// the missing `;` is added so the output is valid v1.
#[test]
#[cfg(feature = "v1")]
fn ensure_v1_adds_semicolon_to_inline_block() {
    let input = "a { b }\n";
    let mut doc = KdlDocument::parse_v2(input).unwrap();
    doc.ensure_v1();
    assert_snapshot!(doc.to_string(), @"a { b; }");
    KdlDocument::parse_v1(&doc.to_string()).expect("ensure_v1 output must parse as v1");
}

// `a {\n b \n}` was multi-line in the source (newline after opening brace),
// so it stays multi-line.
#[test]
fn format_multiline_block_stays_multiline() {
    let input = r#"
a {
    b
}
"#;
    assert_snapshot!(format(input), @r#"
    a {
        b
    }
    "#);
}

// `a { b; c }` (single-line, two children) gets normalized to multi-line:
// `;` is the only legal single-line separator and we consistently choose
// the multi-line form for multi-child blocks.
#[test]
fn format_singleline_block_multiple_children_becomes_multiline() {
    let input = "a { b; c }\n";
    assert_snapshot!(format(input), @r#"
    a {
        b
        c
    }
    "#);
}

// Same shape, parsed as v1.
#[test]
#[cfg(feature = "v1")]
fn format_v1_children_starting_with_comment() {
    let input = r#"
trackpoint {
    // off
    // natural-scroll
}
"#;

    assert_snapshot!(format_v1(input), @r#"
    trackpoint {
        // off
        // natural-scroll
    }
    "#);
}

// v1-routed comment-only children block with a blank line before the
// comment: the blank line is preserved. Pins the `leading_newlines > 1`
// branch of `autoformat_trailing_indented` (uses assert_eq! because
// insta's inline-snapshot normalization swallows leading blank lines
// inside the block).
#[test]
#[cfg(feature = "v1")]
fn format_v1_children_comment_with_leading_blank() {
    let input = "trackpoint {\n\n    // off\n}\n";
    assert_eq!(format_v1(input), "trackpoint {\n\n    // off\n}\n");
}

// Slashdashed block with mis-indented inner content: autoformat preserves
// the original indentation rather than fixing it. This is a deliberate
// "we don't know how to format slashdash, leave it alone" stance, not an
// active formatting choice. Pinned so a future "actually re-indent
// slashdash" change is visible in the diff.
#[test]
fn format_slashdash_block_misindented_input() {
    let input = r#"
/-window-rule {
  geometry-corner-radius 8
      clip-to-geometry "true"
}
"#;
    assert_snapshot!(format(input), @r#"
    /-window-rule {
      geometry-corner-radius 8
          clip-to-geometry "true"
    }
    "#);
}

// BUG: comment lines bracketing a slashdashed block keep their source
// indentation instead of being normalized. The first line is incidentally
// rescued (it's at the very start of the leading decor), but the trailing
// `// trailer` survives with its 8-space mis-indent. No real-world config
// has surfaced this shape yet.
#[test]
fn format_slashdash_block_mixed_with_comments() {
    let input = r#"
    // header
/-window-rule {
        geometry-corner-radius 8
}
        // trailer
node
"#;
    assert_snapshot!(format(input), @r#"
    // header
    /-window-rule {
            geometry-corner-radius 8
    }
            // trailer
    node
    "#);
}

// Slashdashed block whose body happens to contain only `//` lines. The
// whole block round-trips verbatim, including the inner indentation.
#[test]
fn format_slashdash_block_with_inner_comments_only() {
    let input = r#"
/-block {
    // a
    // b
}
"#;
    assert_snapshot!(format(input), @"
    /-block {
        // a
        // b
    }
    ");
}
