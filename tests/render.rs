//! End-to-end tests: run the binary, turn its terminal output back into pixels and decode it.
use image::GrayImage;
use std::process::Command;

const QUIET_ZONE: usize = 4;
const PIXELS_PER_MODULE: u32 = 4;

fn run(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_sina")).args(args).output().expect("binary runs");
    (
        String::from_utf8(out.stdout).expect("utf-8 stdout"),
        String::from_utf8(out.stderr).expect("utf-8 stderr"),
        out.status.success(),
    )
}

/// Reads half-block glyphs back into a module grid, two rows per text line.
fn glyphs_to_image(rendered: &str) -> GrayImage {
    let lines: Vec<&str> = rendered.lines().collect();
    let width = lines[0].chars().count() as u32;
    let height = lines.len() as u32 * 2;
    let mut img = GrayImage::from_pixel(
        width * PIXELS_PER_MODULE,
        height * PIXELS_PER_MODULE,
        image::Luma([255]),
    );
    for (line_index, line) in lines.iter().enumerate() {
        for (col, glyph) in line.chars().enumerate() {
            let (top, bottom) = match glyph {
                '█' => (true, true),
                '▀' => (true, false),
                '▄' => (false, true),
                _ => (false, false),
            };
            for (dark, row) in [(top, line_index * 2), (bottom, line_index * 2 + 1)] {
                if !dark {
                    continue;
                }
                for dy in 0..PIXELS_PER_MODULE {
                    for dx in 0..PIXELS_PER_MODULE {
                        let x = col as u32 * PIXELS_PER_MODULE + dx;
                        let y = row as u32 * PIXELS_PER_MODULE + dy;
                        img.put_pixel(x, y, image::Luma([0]));
                    }
                }
            }
        }
    }
    img
}

fn decode(rendered: &str) -> String {
    let mut prepared = rqrr::PreparedImage::prepare(glyphs_to_image(rendered));
    let grids = prepared.detect_grids();
    assert_eq!(grids.len(), 1, "expected exactly one QR code in the output");
    grids[0].decode().expect("grid decodes").1
}

fn symbol_size(rendered: &str) -> usize {
    rendered.lines().next().expect("output has lines").chars().count() - 2 * QUIET_ZONE
}

#[test]
fn round_trips_every_mode_and_level() {
    let inputs = [
        "0123456789",
        "HELLO WORLD",
        "https://example.com/path?x=1&y=two",
        "Ünïcödé text with émojis 🦀",
        &"7".repeat(1200),
        &"https://example.com/".repeat(20),
    ];
    for text in inputs {
        for level in ["l", "m", "q", "h"] {
            let (stdout, stderr, ok) = run(&["-p", "-e", level, text]);
            assert!(ok, "sina failed for level {level}: {stderr}");
            assert_eq!(decode(&stdout), text, "decode mismatch at level {level}");
        }
    }
}

#[test]
fn picks_the_smallest_version() {
    // 25 alphanumeric characters is the exact capacity of version 1 at level L.
    let fits = "A".repeat(25);
    let (stdout, _, _) = run(&["-p", "-e", "l", &fits]);
    assert_eq!(symbol_size(&stdout), 21);
    let spills = "A".repeat(26);
    let (stdout, _, _) = run(&["-p", "-e", "l", &spills]);
    assert_eq!(symbol_size(&stdout), 25);
}

#[test]
fn reads_text_from_stdin() {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_sina"))
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("binary spawns");
    child.stdin.take().expect("stdin").write_all(b"from stdin\n").expect("write");
    let out = child.wait_with_output().expect("binary exits");
    assert_eq!(decode(&String::from_utf8(out.stdout).expect("utf-8")), "from stdin");
}

#[test]
fn plain_output_matches_snapshot() {
    let (stdout, _, _) = run(&["-p", "HELLO WORLD"]);
    let expected = include_str!("snapshots/hello_world.txt");
    assert_eq!(
        stdout, expected,
        "rendering changed; update tests/snapshots/hello_world.txt if intended"
    );
}

#[test]
fn color_output_wraps_every_line_in_black_on_white() {
    let (stdout, _, _) = run(&["HELLO WORLD"]);
    for line in stdout.lines() {
        assert!(line.starts_with("\x1b[30;47m"), "line lacks color prefix: {line:?}");
        assert!(line.ends_with("\x1b[0m"), "line lacks reset suffix: {line:?}");
    }
    let (plain, _, _) = run(&["-p", "HELLO WORLD"]);
    let stripped: String = stdout.replace("\x1b[30;47m", "").replace("\x1b[0m", "");
    assert_eq!(stripped, plain);
}

#[test]
fn reports_usage_errors() {
    let (_, stderr, ok) = run(&["-e", "x", "text"]);
    assert!(!ok);
    assert!(stderr.contains("unknown error correction level"));
    let (_, stderr, ok) = run(&["--bogus"]);
    assert!(!ok);
    assert!(stderr.contains("unknown option"));
    let (_, stderr, ok) = run(&["-e", "h", &"A".repeat(2000)]);
    assert!(!ok);
    assert!(stderr.contains("too long"));
}
