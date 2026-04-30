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

// BUG: First comment line in a children block is captured as the previous
// node's trailing decor and rendered without the children-block indent,
// producing flush-left output. Subsequent comment lines go through
// autoformat_leading on the next node and get proper indentation. The
// asymmetry is visible above and pinned by this regression test.
#[test]
fn format_comment_only_inside_children() {
    let input = r#"
input {
    // just a comment, no nodes
}
"#;
    assert_snapshot!(format(input), @"
    input {// just a comment, no nodes

    }
    ");
}

// BUG: Multiple top-level comments separated by blank lines collapse the
// blank lines, even though blank lines often carry intentional grouping.
#[test]
fn format_blank_lines_between_top_level_comments() {
    let input = r#"
// first

// second
"#;
    assert_snapshot!(format(input), @r#"
    // first
    // second
    "#);
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
