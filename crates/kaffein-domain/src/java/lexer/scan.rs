//! Character scanners shared by the line-oriented Java lexer.
//!
//! Everything here is a pure function over a single line. Keeping the scanners
//! separate from the state machine makes the lexer easy to test and keeps each
//! file inside the project's file budget.

/// Returns the next character and its UTF-8 width.
pub(super) fn next_char(line: &str, pos: usize) -> Option<(char, usize)> {
    line[pos..].chars().next().map(|ch| (ch, ch.len_utf8()))
}

pub(super) fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphabetic()
}

pub(super) fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphanumeric()
}

pub(super) fn scan_identifier(line: &str, start: usize) -> usize {
    let mut pos = start;
    while let Some((ch, width)) = next_char(line, pos) {
        if !is_ident_continue(ch) {
            break;
        }
        pos += width;
    }
    pos
}

/// Scans a numeric literal starting at `start`.
///
/// The scanner is deliberately permissive: invalid literals are still consumed
/// as a single number so the rest of the line is highlighted normally. It
/// handles decimal, octal, hexadecimal, binary, floating-point, exponent, and
/// suffix forms, including underscores.
pub(super) fn scan_number(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut pos = start;

    if bytes[pos] == b'.' {
        pos += 1;
        while pos < len && is_digit_or_separator(bytes[pos]) {
            pos += 1;
        }
        return scan_suffix(line, scan_exponent(line, pos));
    }

    if bytes[pos] == b'0' && pos + 1 < len {
        match bytes[pos + 1] {
            b'x' | b'X' => {
                let mut cursor = pos + 2;
                let digits = cursor;
                while cursor < len && (bytes[cursor].is_ascii_hexdigit() || bytes[cursor] == b'_') {
                    cursor += 1;
                }
                if cursor == digits {
                    return pos + 1;
                }
                if cursor < len && bytes[cursor] == b'.' {
                    cursor += 1;
                    while cursor < len
                        && (bytes[cursor].is_ascii_hexdigit() || bytes[cursor] == b'_')
                    {
                        cursor += 1;
                    }
                }
                if cursor < len && matches!(bytes[cursor], b'p' | b'P') {
                    cursor = scan_exponent_digits(line, cursor + 1);
                }
                return scan_suffix(line, cursor);
            }
            b'b' | b'B' => {
                let mut cursor = pos + 2;
                let digits = cursor;
                while cursor < len && matches!(bytes[cursor], b'0' | b'1' | b'_') {
                    cursor += 1;
                }
                if cursor == digits {
                    return pos + 1;
                }
                return scan_suffix(line, cursor);
            }
            _ => {}
        }
    }

    while pos < len && is_digit_or_separator(bytes[pos]) {
        pos += 1;
    }
    if pos < len && bytes[pos] == b'.' && bytes.get(pos + 1) != Some(&b'.') {
        pos += 1;
        while pos < len && is_digit_or_separator(bytes[pos]) {
            pos += 1;
        }
    }
    scan_suffix(line, scan_exponent(line, pos))
}

fn is_digit_or_separator(byte: u8) -> bool {
    byte.is_ascii_digit() || byte == b'_'
}

fn scan_exponent(line: &str, pos: usize) -> usize {
    let bytes = line.as_bytes();
    if pos < bytes.len() && matches!(bytes[pos], b'e' | b'E') {
        let cursor = scan_exponent_digits(line, pos + 1);
        if cursor > pos + 1 {
            return cursor;
        }
    }
    pos
}

fn scan_exponent_digits(line: &str, mut pos: usize) -> usize {
    let bytes = line.as_bytes();
    if pos < bytes.len() && matches!(bytes[pos], b'+' | b'-') {
        pos += 1;
    }
    let digits = pos;
    while pos < bytes.len() && is_digit_or_separator(bytes[pos]) {
        pos += 1;
    }
    if pos == digits {
        // No exponent digits: let the caller keep the `e`/`p` as part of the
        // following identifier rather than swallowing it.
        digits.saturating_sub(1)
    } else {
        pos
    }
}

fn scan_suffix(line: &str, pos: usize) -> usize {
    let bytes = line.as_bytes();
    if pos < bytes.len() && matches!(bytes[pos], b'l' | b'L' | b'f' | b'F' | b'd' | b'D') {
        pos + 1
    } else {
        pos
    }
}

/// Scans a quoted string or character literal that must end on this line.
///
/// An unterminated literal ends at the line end. Escapes are skipped by their
/// full UTF-8 width so the scanner never stops inside a character.
pub(super) fn scan_quoted(line: &str, start: usize, quote: u8) -> usize {
    let bytes = line.as_bytes();
    let mut pos = start + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => {
                pos += 1;
                if let Some((_, width)) = next_char(line, pos) {
                    pos += width;
                }
            }
            byte if byte == quote => return pos + 1,
            b'\n' => return pos,
            _ => pos += 1,
        }
    }
    bytes.len()
}

pub(super) fn find_block_comment_end(line: &str, from: usize) -> Option<usize> {
    line[from..].find("*/").map(|index| from + index + 2)
}

/// Returns the byte offset just after the closing `"""`, if it is on this line.
pub(super) fn find_text_block_end(line: &str, from: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut pos = from;
    while pos < bytes.len() {
        if bytes[pos] == b'\\' {
            pos += 1;
            if let Some((_, width)) = next_char(line, pos) {
                pos += width;
            }
            continue;
        }
        if bytes[pos..].starts_with(b"\"\"\"") {
            return Some(pos + 3);
        }
        pos += next_char(line, pos).map_or(1, |(_, width)| width);
    }
    None
}

const OPERATORS: &[&str] = &[
    ">>>=", "<<=", ">>=", ">>>", "->", "::", "==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=",
    "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "=", ">", "<", "!", "~", "?", ":", "+",
    "-", "*", "/", "&", "|", "^", "%",
];

const PUNCTUATION: &[&str] = &["...", "{", "}", "(", ")", "[", "]", ";", ",", "."];

pub(super) fn match_operator(line: &str, pos: usize) -> Option<&'static str> {
    OPERATORS
        .iter()
        .copied()
        .find(|operator| line[pos..].starts_with(operator))
}

pub(super) fn match_punctuation(line: &str, pos: usize) -> Option<&'static str> {
    PUNCTUATION
        .iter()
        .copied()
        .find(|punctuation| line[pos..].starts_with(punctuation))
}
