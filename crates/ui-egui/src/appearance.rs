//! Transient UI-only appearance. The presentation/font database/renderers are never modified.
use crate::omarchy::{Palette, blend};
use crate::theme::Tokens;
use egui::{Color32, FontDefinitions, FontFamily};
use omacraft_ui::{Face, FontOptions, SystemFaces};

/// What the chrome font tokens are built from: the "Use system font" switch and the system faces
/// the appearance worker found (none in tests and on the web).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ChromeFonts {
    pub options: FontOptions,
    pub system: SystemFaces,
}

fn palette_id() -> egui::Id {
    egui::Id::unique("omadeck.appearance.palette")
}
fn fonts_id() -> egui::Id {
    egui::Id::unique("omadeck.appearance.fonts")
}

pub(crate) fn refresh(ctx: &egui::Context, palette: Option<Palette>, fonts: ChromeFonts, allow_fonts: bool) -> (bool, bool) {
    let old_palette = ctx.data(|d| d.get_temp::<Palette>(palette_id()));
    let old_fonts = ctx.data(|d| d.get_temp::<ChromeFonts>(fonts_id())).unwrap_or_default();
    let changed = (old_palette != palette, allow_fonts && old_fonts != fonts);
    if changed.0 {
        ctx.data_mut(|d| {
            if let Some(p) = palette {
                d.insert_temp(palette_id(), p);
            } else {
                d.remove::<Palette>(palette_id());
            }
        });
    }
    if changed.1 {
        ctx.data_mut(|d| d.insert_temp(fonts_id(), fonts));
    }
    changed
}

pub(crate) fn tokens(ctx: &egui::Context, mut t: Tokens) -> Tokens {
    let Some(p) = ctx.data(|d| d.get_temp::<Palette>(palette_id())) else { return t };
    t.dark = p.dark;
    t.chrome = p.background;
    t.card = p.surface;
    t.canvas = p.background;
    t.text = readable(p.foreground, p.surface, 4.5);
    t.text_dim = readable(p.muted, p.surface, 4.5);
    t.text_faint = blend(p.surface, t.text, 0.6);
    t.accent = readable(p.accent, p.surface, 4.5);
    t.accent_text = readable(p.background, t.accent, 4.5);
    t.contextual = t.accent;
    t.text_field = p.background;
    t.field_bg = p.background;
    t.hover = blend(p.surface, t.text, 0.08);
    t.pressed = blend(p.surface, t.text, 0.16);
    t.selected = p.selection;
    t.border = blend(p.surface, t.text, 0.28);
    t.card_border = t.border;
    t.separator = t.border;
    t.control = blend(p.surface, t.text, 0.20);
    t.control_hover = t.hover;
    t.control_pressed = t.pressed;
    // Slide content and paper interaction tokens (caret, guides, handles) retain their baseline.
    t
}

fn luminance(c: Color32) -> f32 {
    let linear = |n: u8| {
        let n = f32::from(n) / 255.0;
        if n <= 0.04045 { n / 12.92 } else { ((n + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
}
fn contrast(a: Color32, b: Color32) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
fn readable(color: Color32, bg: Color32, minimum: f32) -> Color32 {
    if contrast(color, bg) >= minimum {
        return color;
    }
    let target = if contrast(Color32::WHITE, bg) > contrast(Color32::BLACK, bg) { Color32::WHITE } else { Color32::BLACK };
    for i in 1..=20 {
        let c = blend(color, target, i as f32 / 20.0);
        if contrast(c, bg) >= minimum {
            return c;
        }
    }
    target
}

/// Install the chrome font tokens into `defs` from the last [`refresh`].
pub(crate) fn fonts(ctx: &egui::Context, defs: &mut FontDefinitions) {
    let f = ctx.data(|d| d.get_temp::<ChromeFonts>(fonts_id())).unwrap_or_default();
    install(defs, f.options, &f.system);
}

/// OmaCraft font tokens over DeckCraft's interface faces: bundled Inter for chrome
/// (`Proportional`), Inter SemiBold for `bold`, Inter Tabular for numbers, Instrument Sans for
/// display text, and the system mono only for `Monospace`. The craft-fonts script faces follow
/// Inter, so no script draws as boxes. Slide text keeps its own families.
pub(crate) fn install(defs: &mut FontDefinitions, options: FontOptions, system: &SystemFaces) {
    let scripts: Vec<String> = defs
        .families
        .get(&FontFamily::Proportional)
        .map(|list| list.iter().filter(|k| k.starts_with("craft-")).cloned().collect())
        .unwrap_or_default();
    let scripts: Vec<&str> = scripts.iter().map(String::as_str).collect();
    omacraft_ui::fonts::apply(defs, options, system, &scripts);
    // DeckCraft's own bold families follow the strong token.
    let strong = defs.families.get(&Face::UiStrong.family()).cloned().unwrap_or_default();
    defs.families.insert(FontFamily::Name("bold".into()), strong.clone());
    defs.families.insert(FontFamily::Name("omadeck-document-bold".into()), strong);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_changes_chrome_but_keeps_paper_interaction_tokens() {
        let ctx = egui::Context::default();
        let p = Palette::parse("background='#101020'\nforeground='#eeeeff'\naccent='#bbaaee'", false).unwrap();
        assert_eq!(refresh(&ctx, Some(p), ChromeFonts::default(), true), (true, false));
        let base = Tokens::light();
        let result = tokens(&ctx, base);
        assert_eq!(result.chrome, p.background);
        assert_eq!(result.caret, base.caret);
        assert_eq!(result.guide, base.guide);
        assert_eq!(result.handle_fill, base.handle_fill);
        assert!(contrast(result.text, result.card) >= 4.5);
        assert_eq!(refresh(&ctx, None, ChromeFonts::default(), true), (true, false));
        assert_eq!(tokens(&ctx, base).chrome, base.chrome);
    }
    #[test]
    fn chrome_text_styles_resolve_to_inter_never_mono() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply(&ctx, &Tokens::light());
        let mut o = ctx.run_ui(Default::default(), |_| {});
        o.textures_delta.clear();
        let defs = crate::theme::font_definitions(&[]);
        let style = ctx.global_style();
        for text_style in [egui::TextStyle::Body, egui::TextStyle::Button, egui::TextStyle::Small, egui::TextStyle::Heading] {
            let family = &style.text_styles[&text_style].family;
            let first = defs.families[family].first().map(String::as_str);
            assert!(first.is_some_and(|k| k.starts_with("omacraft/Inter")), "{text_style:?} starts with {first:?}");
        }
        for role in
            [omacraft_ui::Role::Menu, omacraft_ui::Role::Body, omacraft_ui::Role::DialogTitle, omacraft_ui::Role::Section, omacraft_ui::Role::Caption]
        {
            assert_eq!(style.text_styles.get(&role.text_style()), Some(&role.font()), "{role:?} is registered");
            assert!(defs.families[&role.font().family][0].starts_with("omacraft/Inter"), "{role:?}");
        }
        for font in [crate::theme::font(13.0), crate::theme::bold(13.0), crate::theme::document_bold(13.0)] {
            assert!(defs.families[&font.family][0].starts_with("omacraft/Inter"), "{font:?}");
        }
        // Proportional, not mono: "iiii" is far narrower than "MMMM" in chrome text.
        let width = |text: &str, font: egui::FontId| ctx.fonts_mut(|f| f.layout_no_wrap(text.into(), font, Color32::WHITE).size().x);
        assert!(width("iiii", omacraft_ui::Role::Menu.font()) < width("MMMM", omacraft_ui::Role::Menu.font()) * 0.6);
        assert!(width("iiii", crate::theme::font(13.0)) < width("MMMM", crate::theme::font(13.0)) * 0.6);
        let mut o = ctx.run_ui(Default::default(), |_| {});
        o.textures_delta.clear();
    }
    #[test]
    fn use_system_font_moves_only_the_ui_tokens() {
        let ctx = egui::Context::default();
        let off = ChromeFonts::default();
        assert!(!off.options.use_system_font, "off by default");
        let on = ChromeFonts { options: FontOptions { use_system_font: true }, system: SystemFaces::default() };
        assert_eq!(refresh(&ctx, None, on.clone(), true), (false, true));
        assert_eq!(refresh(&ctx, None, on.clone(), true), (false, false), "no reinstall without a change");
        assert_eq!(refresh(&ctx, None, off.clone(), false), (false, false), "deferred while input owns the pass");
        let mut defs = crate::theme::font_definitions(&[]);
        fonts(&ctx, &mut defs);
        // With no system faces found, Inter stays the UI face; display never changes.
        assert_eq!(defs.families[&FontFamily::Proportional][0], omacraft_ui::fonts::keys::INTER);
        assert_eq!(defs.families[&Face::Display.family()][0], omacraft_ui::fonts::keys::DISPLAY);
    }
}
