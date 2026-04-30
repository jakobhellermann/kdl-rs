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
    assert_eq!(
        fmt,
        r#"a {
    b {
        c {
        }
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
// parser stores `/-key="hidden"` in the next entry's `leading` field, and
// `KdlEntry::autoformat` clears `format = None` to canonicalize spacing —
// taking the slashdashed content with it. Fixing this likely needs the
// slashdashed entry to be modeled as its own KdlEntry (with a commented-
// out flag) rather than buried in another entry's whitespace.
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
    let input = "outer {\n    inner\n}\n";
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
        Mod+o hotkey-overlay-title="null" {
            show-hotkey-overlay
        }
    }
    "#);
}
