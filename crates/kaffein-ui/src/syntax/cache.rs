//! Incremental token cache and galley cache for the editor surface.
//!
//! Tokenization is line-oriented. On every text change the cache compares line
//! hashes, re-lexes only the changed middle, and reuses the unchanged prefix
//! and suffix when the lexical state still matches. The resulting galley is
//! cached as well, so an unchanged frame does no lexing and no font layout.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use egui::{Galley, TextStyle, Ui};
use kaffein_domain::{tokenize_line, LexState, Token};

use super::layout;
use super::theme::SyntaxTheme;

#[derive(Debug, Default)]
pub(crate) struct HighlightCache {
    lines: Vec<LineCache>,
    generation: Rc<Cell<u64>>,
    applied_generation: Option<u64>,
    galley: Option<CachedGalley>,
    last_lexed_lines: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LineCache {
    pub(crate) start: usize,
    pub(crate) content_end: usize,
    pub(crate) end: usize,
    pub(crate) hash: u64,
    pub(crate) state_before: LexState,
    pub(crate) state_after: LexState,
    pub(crate) tokens: Vec<Token>,
}

#[derive(Clone, Copy, Debug)]
struct NewLine {
    start: usize,
    content_end: usize,
    end: usize,
    hash: u64,
}

#[derive(Debug)]
struct CachedGalley {
    generation: u64,
    wrap_width_bits: u32,
    pixels_per_point_bits: u32,
    syntax: SyntaxTheme,
    galley: Arc<Galley>,
}

impl HighlightCache {
    /// A handle the editor buffer uses to report mutations.
    pub(crate) fn generation(&self) -> Rc<Cell<u64>> {
        Rc::clone(&self.generation)
    }

    /// Drops every line and galley, for example when switching documents.
    pub(crate) fn reset(&mut self) {
        self.lines.clear();
        self.applied_generation = None;
        self.galley = None;
        self.bump();
    }

    /// Forces the next layout to compare the text again, for example after the
    /// document revision changed. Unchanged text keeps its cached galley.
    pub(crate) fn invalidate(&mut self) {
        self.applied_generation = None;
        self.bump();
    }

    pub(crate) fn layout(
        &mut self,
        ui: &Ui,
        text: &str,
        wrap_width: f32,
        generation: u64,
        syntax: &SyntaxTheme,
    ) -> Arc<Galley> {
        let pixels_per_point = ui.ctx().pixels_per_point();
        let changed = self.update(text, generation);

        if !changed {
            if let Some(cached) = &mut self.galley {
                if cached.wrap_width_bits == wrap_width.to_bits()
                    && cached.pixels_per_point_bits == pixels_per_point.to_bits()
                    && cached.syntax == *syntax
                {
                    cached.generation = generation;
                    return Arc::clone(&cached.galley);
                }
            }
        }

        let font_id = ui.style().text_styles[&TextStyle::Monospace].clone();
        let job = layout::build_job(text, &self.lines, font_id, wrap_width, syntax);
        let galley = ui.fonts(|fonts| fonts.layout_job(job));
        self.galley = Some(CachedGalley {
            generation,
            wrap_width_bits: wrap_width.to_bits(),
            pixels_per_point_bits: pixels_per_point.to_bits(),
            syntax: *syntax,
            galley: Arc::clone(&galley),
        });
        galley
    }

    fn bump(&self) {
        self.generation.set(self.generation.get().wrapping_add(1));
    }

    /// Returns `true` when the line cache changed and the galley must be rebuilt.
    fn update(&mut self, text: &str, generation: u64) -> bool {
        if self.applied_generation == Some(generation) {
            return false;
        }
        self.applied_generation = Some(generation);
        self.last_lexed_lines = 0;

        let new_lines = split_lines(text);
        if self.lines.len() == new_lines.len()
            && self.lines.iter().zip(&new_lines).all(|(old, new)| {
                old.hash == new.hash
                    && old.start == new.start
                    && old.content_end == new.content_end
                    && old.end == new.end
            })
        {
            return false;
        }

        self.rebuild(text, &new_lines);
        self.galley = None;
        true
    }

    fn rebuild(&mut self, text: &str, new_lines: &[NewLine]) {
        let old_lines = std::mem::take(&mut self.lines);
        let old_len = old_lines.len();
        let new_len = new_lines.len();

        let mut prefix = 0;
        while prefix < old_len.min(new_len) && old_lines[prefix].hash == new_lines[prefix].hash {
            prefix += 1;
        }

        let mut suffix = 0;
        while suffix
            < old_len
                .saturating_sub(prefix)
                .min(new_len.saturating_sub(prefix))
            && old_lines[old_len - 1 - suffix].hash == new_lines[new_len - 1 - suffix].hash
        {
            suffix += 1;
        }

        let mut result = Vec::with_capacity(new_len);
        let mut old_lines = old_lines.into_iter();

        for new_line in &new_lines[..prefix] {
            if let Some(old_line) = old_lines.next() {
                result.push(LineCache {
                    start: new_line.start,
                    content_end: new_line.content_end,
                    end: new_line.end,
                    hash: new_line.hash,
                    ..old_line
                });
            }
        }

        let middle_old = old_len - prefix - suffix;
        for _ in 0..middle_old {
            old_lines.next();
        }
        let suffix_old: Vec<LineCache> = old_lines.collect();

        let middle_new = new_len - prefix - suffix;
        let mut state = result
            .last()
            .map_or(LexState::Normal, |line| line.state_after);

        for new_line in &new_lines[prefix..prefix + middle_new] {
            let state_before = state;
            let lexed = tokenize_line(&text[new_line.start..new_line.content_end], state_before);
            state = lexed.end_state;
            self.last_lexed_lines += 1;
            result.push(LineCache {
                start: new_line.start,
                content_end: new_line.content_end,
                end: new_line.end,
                hash: new_line.hash,
                state_before,
                state_after: state,
                tokens: lexed.tokens,
            });
        }

        let suffix_states_match = suffix_old
            .first()
            .is_none_or(|line| line.state_before == state);

        if suffix_states_match {
            for (old_line, new_line) in suffix_old
                .into_iter()
                .zip(&new_lines[prefix + middle_new..])
            {
                result.push(LineCache {
                    start: new_line.start,
                    content_end: new_line.content_end,
                    end: new_line.end,
                    hash: new_line.hash,
                    ..old_line
                });
            }
        } else {
            for new_line in &new_lines[prefix + middle_new..] {
                let state_before = state;
                let lexed =
                    tokenize_line(&text[new_line.start..new_line.content_end], state_before);
                state = lexed.end_state;
                self.last_lexed_lines += 1;
                result.push(LineCache {
                    start: new_line.start,
                    content_end: new_line.content_end,
                    end: new_line.end,
                    hash: new_line.hash,
                    state_before,
                    state_after: state,
                    tokens: lexed.tokens,
                });
            }
        }

        self.lines = result;
    }

    #[cfg(test)]
    pub(crate) fn relex_for_test(&mut self, text: &str) -> usize {
        self.bump();
        let generation = self.generation.get();
        self.update(text, generation);
        self.last_lexed_lines
    }

    #[cfg(test)]
    pub(crate) fn lines_for_test(&self) -> &[LineCache] {
        &self.lines
    }

    #[cfg(test)]
    pub(crate) fn line_count_for_test(&self) -> usize {
        self.lines.len()
    }
}

fn split_lines(text: &str) -> Vec<NewLine> {
    let mut lines = Vec::new();
    let mut start = 0;

    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            lines.push(NewLine {
                start,
                content_end: index,
                end: index + 1,
                hash: line_hash(&text[start..index]),
            });
            start = index + 1;
        }
    }

    if start < text.len() {
        lines.push(NewLine {
            start,
            content_end: text.len(),
            end: text.len(),
            hash: line_hash(&text[start..]),
        });
    }

    lines
}

fn line_hash(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
