use calc_core::lexer::{tokenize, TokenKind};
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Number,
    Str,
    Func,
    Ident,
    Op,
    Keyword,
}

pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
} // char indices

pub fn spans(src: &str) -> Vec<Span> {
    let toks = match tokenize(src) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let kind = match &t.kind {
            TokenKind::Int(_) | TokenKind::Float(_) => Kind::Number,
            TokenKind::Str(_) => Kind::Str,
            TokenKind::True
            | TokenKind::False
            | TokenKind::KwFn
            | TokenKind::KwAlias
            | TokenKind::KwWhile
            | TokenKind::KwRepeat => Kind::Keyword,
            TokenKind::Ident(_) => {
                if matches!(toks.get(i + 1).map(|n| &n.kind), Some(TokenKind::LParen)) {
                    Kind::Func
                } else {
                    Kind::Ident
                }
            }
            TokenKind::Eof | TokenKind::Newline => continue,
            _ => Kind::Op,
        };
        out.push(Span {
            start: t.pos,
            end: t.end,
            kind,
        });
    }
    out
}

fn color(k: Kind) -> Color32 {
    match k {
        Kind::Number => Color32::from_rgb(0x2c, 0x7b, 0xd6),
        Kind::Str => Color32::from_rgb(0x2e, 0x8b, 0x4e),
        Kind::Func => Color32::from_rgb(0x8a, 0x4f, 0xc7),
        Kind::Keyword => Color32::from_rgb(0xc7, 0x6b, 0x1f),
        Kind::Op => Color32::from_rgb(0x88, 0x88, 0x88),
        Kind::Ident => Color32::from_rgb(0xcc, 0xcc, 0xcc),
    }
}

pub fn layout_job(text: &str, font_size: f32) -> LayoutJob {
    let font = FontId::monospace(font_size);
    let default = Color32::from_gray(200);
    let mut job = LayoutJob::default();
    let chars: Vec<char> = text.chars().collect();
    let sp = spans(text);
    let mut idx = 0usize;
    let fmt = |c: Color32| TextFormat {
        font_id: font.clone(),
        color: c,
        ..Default::default()
    };
    let slice = |a: usize, b: usize| chars[a..b].iter().collect::<String>();
    for s in sp {
        if s.start > idx {
            job.append(&slice(idx, s.start), 0.0, fmt(default));
        }
        let a = s.start.max(idx);
        if s.end > a {
            job.append(&slice(a, s.end), 0.0, fmt(color(s.kind)));
            idx = s.end;
        }
    }
    if idx < chars.len() {
        job.append(&slice(idx, chars.len()), 0.0, fmt(default));
    }
    job
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_tokens() {
        let kinds: Vec<Kind> = spans("Sqrt(2) + 0x1F").into_iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&Kind::Func));
        assert!(kinds.contains(&Kind::Number));
        assert!(kinds.contains(&Kind::Op));
    }
    #[test]
    fn bad_input_does_not_panic() {
        let _ = spans("\"незакрытая");
        let _ = spans("0xZZ");
    }
}
