//! Builds the egui text layout for one document from cached line tokens.

use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};

use super::cache::LineCache;
use super::theme::SyntaxTheme;

pub(crate) fn build_job(
    text: &str,
    lines: &[LineCache],
    font_id: FontId,
    wrap_width: f32,
    syntax: &SyntaxTheme,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    for line in lines {
        let line_text = &text[line.start..line.content_end];
        let mut cursor = 0;
        for token in &line.tokens {
            let start = token.start.clamp(cursor, line_text.len());
            let end = token.end.clamp(start, line_text.len());
            if start > cursor {
                append(
                    &mut job,
                    &line_text[cursor..start],
                    syntax.identifier,
                    &font_id,
                );
            }
            if end > start {
                append(
                    &mut job,
                    &line_text[start..end],
                    syntax.color(token.kind),
                    &font_id,
                );
            }
            cursor = end;
        }
        if cursor < line_text.len() {
            append(&mut job, &line_text[cursor..], syntax.identifier, &font_id);
        }
        if line.end > line.content_end {
            append(&mut job, "\n", syntax.identifier, &font_id);
        }
    }

    job
}

fn append(job: &mut LayoutJob, text: &str, color: Color32, font_id: &FontId) {
    if text.is_empty() {
        return;
    }
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font_id.clone(),
            color,
            ..Default::default()
        },
    );
}
