//! Title bar, ribbon tabs and the ribbon card with every tab's groups.

use deckcraft_engine::Target;
use deckcraft_model::{ShapeId, ShapeKind, anim::AnimClass};
use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Value, json};

use crate::SlideApp;
use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};
use crate::widgets::{
    self, RIBBON_H, big_button, big_button_ex, color_split, dropdown, group, icon_toggle, rows, small_button, small_button_ex, small_button_named,
};

mod animations;
mod arrange;
mod chart;
mod design;
mod draw;
mod galleries;
mod home;
mod insert;
mod masters;
mod picture_format;
mod playback;
mod review;
mod shape_format;
mod slide_show;
mod smartart;
mod table;
mod title_bar;
mod transitions;
mod view;

use animations::animations;
pub use animations::paint_anim_tile;
#[cfg(test)]
use animations::{animation_effect_rows, marked_options, text_build_id};
pub use arrange::{align_menu, arrange_menu};
use chart::chart_design;
use design::design;
pub use design::paint_theme_tile;
use draw::draw;
pub use galleries::{cref_param, effects_menu, outline_menu, paint_preset, shapes_gallery};
pub use home::font_families;
use home::home;
use insert::insert;
pub use insert::table_grid_picker;
use masters::{handout_master, notes_master, slide_master};
use picture_format::picture_format;
use playback::playback;
use review::review;
use shape_format::shape_format;
use slide_show::{record, slide_show};
use smartart::smartart_design;
pub use table::paint_table_style;
use table::{table_design, table_layout};
pub use title_bar::{MAC_CONTENT_LEFT, MAC_TITLE_BAR, TITLE_H, title_bar};
#[cfg(test)]
use title_bar::{MAC_LIGHT, MAC_LIGHT_PITCH, MAC_LIGHTS_LEFT, search_width, title_galleys, title_geometry, title_max_width};
pub use transitions::paint_transition_tile;
#[cfg(test)]
use transitions::transition_effect_rows;
use transitions::transitions;
use view::view;

/// Is engine command `id` enabled right now?
/// Whether `id` names an engine or UI command.
pub fn known(id: &str) -> bool {
    deckcraft_engine::find_command(id).is_some() || crate::UI_COMMANDS.iter().any(|c| c.0 == id)
}

pub fn enabled(app: &SlideApp, id: &str) -> bool {
    match deckcraft_engine::find_command(id) {
        Some(c) => (c.enabled)(&app.session).is_ok(),
        None => crate::UI_COMMANDS.iter().any(|c| c.0 == id),
    }
}

fn run(app: &mut SlideApp, id: &str, p: Value) {
    let _ = app.run(id, p);
}

/// Which contextual tabs apply to the selection.
fn contextual(app: &SlideApp) -> Vec<&'static str> {
    let mut v = vec![];
    let Some(st) = app.session.active() else { return v };
    if st.selection.target == Target::Handout {
        v.push("Handout Master");
    } else if st.selection.target == Target::NotesMaster {
        v.push("Notes Master");
    } else if st.selection.target != Target::Slides {
        v.push("Slide Master");
    }
    let shapes = st.selected_shapes();
    if shapes.iter().any(|s| matches!(s.kind, ShapeKind::Shape | ShapeKind::Group { .. } | ShapeKind::Connector { .. }))
        || st.selection.text.is_some()
    {
        v.push("Shape Format");
    }
    if shapes.iter().any(|s| matches!(s.kind, ShapeKind::Picture { .. })) {
        v.push("Picture Format");
    }
    if shapes.iter().any(|s| matches!(s.kind, ShapeKind::Table(_))) || st.selection.text.as_ref().is_some_and(|t| t.cell.is_some()) {
        v.push("Table Design");
        v.push("Layout");
    }
    if deckcraft_engine::cmd::smartart::selected(st).is_some() {
        v.push("SmartArt Design");
    }
    if shapes.iter().any(|s| matches!(s.kind, ShapeKind::Chart(_))) {
        v.push("Chart Design");
    }
    if shapes.iter().any(|s| matches!(s.kind, ShapeKind::Media(_))) {
        v.push("Playback");
    }
    v
}

/// Tab strip geometry and type, matching OmaGrid (docs/ui-consistency.md): a 34 pt row whose tabs
/// are 30 pt hit-boxes, labels 13.5 pt padded by 22 pt, 2 pt apart, starting 18 pt in.
const TAB_ROW_H: f32 = 34.0;
const TAB_HIT_H: f32 = 30.0;
const TAB_FONT: f32 = 13.5;
const TAB_PAD: f32 = 22.0;
const TAB_SPACING: f32 = 2.0;
const TAB_INSET: f32 = 18.0;
/// The ribbon body under the tab row (card plus margins) and its card, as OmaGrid's.
const BODY_H: f32 = 92.0;
const CARD_H: f32 = 88.0;
const _: () = assert!(CARD_H - 12.0 == RIBBON_H, "the card's padded content is the ribbon content height");

const TABS: [&str; 10] = ["Home", "Insert", "Draw", "Design", "Transitions", "Animations", "Slide Show", "Record", "Review", "View"];

/// Tab row and the ribbon card.
pub fn show(app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx_tabs = contextual(app);
    if !TABS.contains(&app.ui.tab.as_str()) && !ctx_tabs.contains(&app.ui.tab.as_str()) {
        app.ui.tab = "Home".into();
    }
    let h = if app.ui.ribbon_collapsed { TAB_ROW_H } else { TAB_ROW_H + BODY_H };
    egui::Panel::top("ribbon").exact_size(h).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let rect = ui.max_rect();
        // Tabs, laid out as OmaGrid's strip (docs/ui-consistency.md). File comes first and opens
        // the File tab's full-window view (Backstage), as in PowerPoint.
        let mut x = rect.min.x + TAB_INSET;
        let y = rect.min.y;
        {
            let font = theme::font(TAB_FONT);
            let w = (ui.fonts_mut(|f| f.layout_no_wrap("File".to_string(), font.clone(), t.text).size().x) + TAB_PAD).max(46.0);
            let r = Rect::from_min_size(pos2(x, y), vec2(w, TAB_HIT_H));
            let resp = ui.interact(r, ui.scope_id().with("tab_file"), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "File"));
            if resp.hovered() {
                ui.painter().rect_filled(r.shrink2(vec2(2.0, 4.0)), CornerRadius::same(5), t.hover);
            }
            ui.painter().text(r.center(), Align2::CENTER_CENTER, "File", font, t.accent);
            if resp.on_hover_text("File (Alt+F)").clicked() {
                app.open_backstage(crate::backstage::Page::Home);
            }
            x += w + TAB_SPACING;
        }
        let all: Vec<(&str, bool)> = TABS.iter().map(|s| (*s, false)).chain(ctx_tabs.iter().map(|s| (*s, true))).collect();
        for (name, contextual) in all {
            let font = theme::font(TAB_FONT);
            let tw = ui.fonts_mut(|f| f.layout_no_wrap(name.to_string(), font.clone(), t.text).size().x);
            let w = tw + TAB_PAD;
            let r = Rect::from_min_size(pos2(x, y), vec2(w, TAB_HIT_H));
            let resp = ui.interact(r, ui.scope_id().with(("tab", name)), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
            let active = app.ui.tab == name;
            if resp.hovered() && !active {
                ui.painter().rect_filled(r.shrink2(vec2(2.0, 4.0)), CornerRadius::same(5), t.hover);
            }
            let col = if contextual {
                t.contextual
            } else if active {
                t.text
            } else {
                t.text_dim
            };
            let f = if active { theme::bold(TAB_FONT) } else { font };
            ui.painter().text(r.center(), Align2::CENTER_CENTER, name, f, col);
            if active {
                let ul = Rect::from_center_size(pos2(r.center().x, r.max.y - 3.0), vec2(tw.clamp(20.0, 40.0), 3.0));
                ui.painter().rect_filled(ul, 1.5, t.accent);
            }
            if resp.clicked() {
                if active {
                    app.ui.ribbon_collapsed = !app.ui.ribbon_collapsed;
                } else {
                    app.ui.tab = name.to_string();
                    app.ui.ribbon_collapsed = false;
                }
            }
            if resp.double_clicked() {
                app.ui.ribbon_collapsed = !app.ui.ribbon_collapsed;
            }
            x += w + TAB_SPACING;
        }
        if app.ui.ribbon_collapsed {
            return;
        }
        // The ribbon card, as OmaGrid's: 88 pt high under the 30 pt tabs (2 pt below them), 10 pt
        // in from the panel edges, radius 10 with a 1 pt border, content padded (10, 6).
        let card = Rect::from_min_size(pos2(rect.min.x + 10.0, rect.min.y + TAB_HIT_H + 2.0), vec2((rect.width() - 20.0).max(0.0), CARD_H));
        ui.painter().rect_filled(card, CornerRadius::same(10), t.card);
        ui.painter().rect_stroke(card, CornerRadius::same(10), Stroke::new(1.0, t.card_border), egui::StrokeKind::Inside);
        let inner = card.shrink2(vec2(10.0, 6.0));
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        cui.set_clip_rect(inner.expand2(vec2(0.0, 3.0)));
        // Reserve a gutter on both sides for the overflow chevrons. They are painted and
        // interacted in that gutter, never on top of the first/last visible button: the bar they
        // reveal must not steal the click they were meant to make reachable.
        // The left gutter only once the row has been scrolled (last frame's offset), so an
        // unscrolled row starts at the card's padding, as OmaGrid's does.
        let edge = 18.0_f32.min(inner.width() * 0.25);
        let scrolled = app.ribbon_scroll.get(&app.ui.tab).is_some_and(|bits| f32::from_bits(*bits) > 1.0);
        let left_edge = if scrolled { edge } else { 0.0 };
        let scroll_rect = Rect::from_min_max(pos2(inner.left() + left_edge, inner.top()), pos2(inner.right() - edge, inner.bottom()));
        let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(scroll_rect));
        sui.set_clip_rect(scroll_rect.expand2(vec2(0.0, 3.0)));
        let tab = app.ui.tab.clone();
        // The card used to clip a too-wide group row with no way to reach the rest. A horizontal
        // scroll area plus edge chevrons keeps every group reachable at the default 1440 px
        // window, including Home's Editing group (Replace Fonts…). The bar is hidden; the
        // chevrons are the visible overflow control and are accessible buttons of their own.
        let request_id = cui.make_persistent_id(("ribbon_scroll_request", tab.as_str()));
        let requested = cui.ctx().data_mut(|d| d.remove_temp::<f32>(request_id));
        let mut area = egui::ScrollArea::horizontal().id_salt(("ribbon_scroll", tab.as_str()));
        if let Some(offset) = requested {
            area = area.horizontal_scroll_offset(offset);
        }
        let scroll =
            area.scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).auto_shrink([false, false]).show(&mut sui, |ui| {
                ui.set_min_height(RIBBON_H);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    match tab.as_str() {
                        "Home" => home(app, ui),
                        "Insert" => insert(app, ui),
                        "Draw" => draw(app, ui),
                        "Design" => design(app, ui),
                        "Transitions" => transitions(app, ui),
                        "Animations" => animations(app, ui),
                        "Slide Show" => slide_show(app, ui),
                        "Record" => record(app, ui),
                        "Review" => review(app, ui),
                        "View" => view(app, ui),
                        "Shape Format" => shape_format(app, ui),
                        "Picture Format" => picture_format(app, ui),
                        "Table Design" => table_design(app, ui),
                        "Layout" => table_layout(app, ui),
                        "Chart Design" => chart_design(app, ui),
                        "SmartArt Design" => smartart_design(app, ui),
                        "Playback" => playback(app, ui),
                        "Slide Master" => slide_master(app, ui),
                        "Handout Master" => handout_master(app, ui),
                        "Notes Master" => notes_master(app, ui),
                        _ => {}
                    }
                });
            });
        let r = scroll.inner_rect;
        // A gallery sized from this reads it on the next frame, so a change (a resize, a tab
        // shown for the first time) asks for one; within half a point it is the same layout.
        let spare = r.width() - scroll.content_size.x;
        let moved = ui.ctx().data_mut(|d| {
            let was = d.get_temp::<f32>(spare_room_id(&tab));
            d.insert_temp(spare_room_id(&tab), spare);
            was.is_none_or(|was| (was - spare).abs() > 0.5)
        });
        if moved {
            ui.ctx().request_repaint();
        }
        let overflow = (scroll.content_size.x - r.width()).max(0.0);
        let offset = scroll.state.clamped_offset().x;
        // Record the rendered offset for `InputTarget`: the same window point hits a different
        // button when the ribbon scrolls between queueing and delivering a control click.
        app.ribbon_scroll.insert(tab.clone(), offset.to_bits());
        let step = 160.0;
        let mut next = None;
        if offset > 1.0 {
            let left = Rect::from_min_max(pos2((r.left() - edge).max(inner.left()), r.top()), pos2(r.left().max(inner.left() + edge), r.bottom()));
            if edge_cue(&mut cui, left, "‹", "Scroll ribbon left", &t) {
                next = Some(offset - step);
            }
        }
        if overflow - offset > 1.0 {
            let right = Rect::from_min_max(pos2(r.right(), r.top()), pos2((r.right() + edge).min(inner.right()), r.bottom()));
            if edge_cue(&mut cui, right, "›", "Scroll ribbon right", &t) {
                next = Some(offset + step);
            }
        }
        if let Some(x) = next {
            ui.ctx().data_mut(|d| d.insert_temp(request_id, x.clamp(0.0, overflow)));
            ui.ctx().request_repaint();
        }
    });
}

fn spare_room_id(tab: &str) -> egui::Id {
    egui::Id::unique(("ribbon_spare_room", tab))
}

/// The width the tab's groups left free in the ribbon on the last frame it was drawn; negative
/// when they ran past its edge. A gallery that gives way to the groups after it sizes itself
/// from this.
pub(super) fn spare_room(ctx: &egui::Context, tab: &str) -> Option<f32> {
    ctx.data(|d| d.get_temp::<f32>(spare_room_id(tab)))
}

/// A chevron over a ribbon edge that scrolls it; `true` on the frame it is clicked.
fn edge_cue(ui: &mut Ui, rect: Rect, glyph: &str, hint: &str, t: &Tokens) -> bool {
    let resp = ui.interact(rect, ui.scope_id().with(glyph), Sense::click()).on_hover_text(hint);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, hint));
    // OmaGrid's scroll arrow: a small bordered tab with a chevron, hover-filled.
    ui.painter().rect_filled(rect, CornerRadius::same(4), if resp.hovered() { t.hover } else { t.card });
    ui.painter().rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0, t.card_border), egui::StrokeKind::Inside);
    let icon = if glyph == "‹" { Icon::ChevronLeft } else { Icon::ChevronRight };
    icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(14.0, 14.0)), icon, t.text, false);
    resp.clicked()
}

/// Draw a large ribbon command button, honour its enabled state, run the command on click, and
/// attach its menu.
fn big(app: &mut SlideApp, ui: &mut Ui, icon: Icon, label: &str, id: &str, p: Value) {
    let en = enabled(app, id);
    let r = big_button(ui, icon, label, en);
    if id == "insert.dateTime" {
        let name = label.replace('\n', " ");
        r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, en, &name));
    }
    let r = r.on_hover_text(tip(id, label));
    if r.clicked() {
        run(app, id, p.clone());
    }
    crate::qat::ribbon_menu(app, &r, icon, label, id, &p);
}

fn small(app: &mut SlideApp, ui: &mut Ui, icon: Icon, label: &str, id: &str, p: Value) {
    let en = enabled(app, id);
    let r = small_button(ui, icon, label, en, false).on_hover_text(tip(id, label));
    if r.clicked() {
        run(app, id, p.clone());
    }
    crate::qat::ribbon_menu(app, &r, icon, label, id, &p);
}

fn toggle(app: &mut SlideApp, ui: &mut Ui, icon: Icon, label: &str, id: &str, on: bool) {
    let en = enabled(app, id);
    if icon_toggle(ui, icon, &tip(id, label), en, on).clicked() {
        run(app, id, json!({}));
    }
}

fn tip(id: &str, label: &str) -> String {
    let sc = deckcraft_engine::find_command(id).and_then(|c| c.shortcut).or_else(|| crate::UI_COMMANDS.iter().find(|c| c.0 == id).and_then(|c| c.2));
    match sc {
        Some(s) => format!("{label} ({})", pretty_shortcut(s)),
        None => label.to_string(),
    }
}

pub fn pretty_shortcut(s: &str) -> String {
    if cfg!(target_os = "macos") {
        s.replace("Cmd+", "⌘").replace("Shift+", "⇧").replace("Alt+", "⌥").replace("Ctrl+", "⌃")
    } else {
        s.replace("Cmd+", "Ctrl+")
    }
}

fn scheme(app: &SlideApp) -> deckcraft_color::ColorScheme {
    app.session.active().and_then(|d| d.doc.masters.first().map(|m| m.scheme())).unwrap_or_else(|| deckcraft_model::Theme::default().colors)
}

/// The formatting state at the selection (bold, size…).
pub fn fmt_state(app: &mut SlideApp) -> Value {
    if app.session.active().is_none_or(|d| d.selection.shapes.is_empty() && d.selection.text.is_none()) {
        return json!({});
    }
    app.session.execute("format.state", &json!({})).unwrap_or_default()
}

pub fn menu_item(app: &mut SlideApp, ui: &mut Ui, icon: Icon, label: &str, id: &str, p: Value) {
    let en = enabled(app, id);
    let r = small_button(ui, icon, label, en, false);
    if r.clicked() {
        run(app, id, p);
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deckcraft_model::anim::TextBuild;

    /// Paints one transition tile and one animation tile and returns the colour of every label.
    fn tile_label_colours(dark: bool) -> Vec<Color32> {
        let ctx = egui::Context::default();
        ctx.set_visuals(if dark { egui::Visuals::dark() } else { egui::Visuals::light() });
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(80.0, 60.0));
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            paint_transition_tile(ui.painter(), r, ("fade", "Fade"));
            paint_anim_tile(ui.painter(), r, "Appear", AnimClass::Entrance, true);
        });
        out.textures_delta.clear();
        out.shapes
            .iter()
            .flat_map(|c| match &c.shape {
                egui::Shape::Text(s) => s.galley.job.sections.iter().map(|x| x.format.color).collect(),
                _ => Vec::new(),
            })
            .collect()
    }

    #[test]
    fn ribbon_tile_labels_follow_the_theme() {
        // #159: the transition and animation tiles were always painted with the light palette,
        // so their labels were dark grey on the dark ribbon.
        let dark = tile_label_colours(true);
        assert_eq!(dark, vec![Tokens::dark().text; 2]);
        let light = tile_label_colours(false);
        assert_eq!(light, vec![Tokens::light().text; 2]);
    }

    fn marked(rows: Vec<(&str, bool)>) -> Vec<&str> {
        rows.into_iter().filter(|(_, on)| *on).map(|(id, _)| id).collect()
    }

    #[test]
    fn effect_options_mark_only_the_stored_choice() {
        assert_eq!(marked(transition_effect_rows("push", Some("l"))), vec!["l"]);
        assert_eq!(transition_effect_rows("push", Some("l")).into_iter().map(|(_, on)| on).collect::<Vec<_>>(), vec![false, false, true, false]);
        assert_eq!(marked(transition_effect_rows("fade", Some("throughBlack"))), vec!["throughBlack"]);
        assert!(marked(transition_effect_rows("push", Some("missing"))).is_empty());
        assert!(transition_effect_rows("none", None).is_empty());
        assert_eq!(marked(transition_effect_rows("cut", Some(""))), vec![""]);
        assert_eq!(marked(animation_effect_rows("fly", AnimClass::Entrance, Some("l"))), vec!["l"]);
        assert!(marked(animation_effect_rows("fly", AnimClass::Entrance, Some("nope"))).is_empty());
        assert!(animation_effect_rows("appear", AnimClass::Entrance, Some("l")).is_empty());
        assert_eq!(marked(marked_options(&["asOne", "byParagraph"], Some(text_build_id(TextBuild::ByParagraph)))), vec!["byParagraph"]);
        assert_eq!(marked(marked_options(&["asOne", "byParagraph"], Some(text_build_id(TextBuild::AsOne)))), vec!["asOne"]);
        assert!(marked(marked_options(&["asOne", "byParagraph"], Some(text_build_id(TextBuild::ByWord)))).is_empty());
    }
}

#[cfg(test)]
mod effects_tests;
pub mod qat;

#[cfg(test)]
mod master_layout_tests;

#[cfg(test)]
mod icons_popup_tests;

#[cfg(test)]
mod add_animation_menu_tests;

#[cfg(test)]
mod align_tests;

#[cfg(test)]
mod title_bar_tests {
    use super::*;

    #[test]
    fn mac_title_bar_centres_and_clears_the_traffic_lights() {
        let (h, left, control) = title_geometry(true);
        // AppKit centres the lights in its own band, so the bar is that band.
        assert_eq!(h, MAC_TITLE_BAR);
        // Content keeps the same gap after the lights as the lights keep from the top.
        let lights_end = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT;
        assert_eq!(left - lights_end, (h - MAC_LIGHT) / 2.0);
        // The tallest controls keep at least 4 pt above and below.
        assert!(control <= h - 8.0);
        // Other platforms keep their system frame and the original bar.
        assert_eq!(title_geometry(false), (TITLE_H, 10.0, 26.0));
    }

    #[test]
    fn search_box_shrinks_then_collapses_to_an_icon() {
        assert_eq!(search_width(500.0), 260.0, "full width when there is room");
        assert_eq!(search_width(200.0), 200.0, "shrinks to the room left beside the title");
        assert_eq!(search_width(140.0), 140.0);
        assert_eq!(search_width(139.0), 28.0, "below its minimum it becomes an icon button");
        assert_eq!(search_width(-50.0), 28.0);
    }

    #[test]
    fn title_fits_between_the_controls() {
        // Room limited by the nearer side, symmetric about the centre, never negative.
        assert_eq!(title_max_width(500.0, 300.0, 900.0), 2.0 * (200.0 - 12.0));
        assert_eq!(title_max_width(500.0, 100.0, 650.0), 2.0 * (150.0 - 12.0));
        assert_eq!(title_max_width(500.0, 520.0, 900.0), 0.0);
    }

    /// Lay out a title in a real egui frame, returning (glyphs shown, title width, marker width).
    fn layout_title(title: &str, dirty: bool, max_w: f32) -> (String, f32, f32) {
        let ctx = egui::Context::default();
        theme::install_fonts(&ctx);
        let mut out = (String::new(), 0.0, 0.0);
        let mut frame = ctx.run_ui(egui::RawInput::default(), |ui| {
            let (galley, dot) = title_galleys(ui.painter(), title, dirty, max_w, Color32::WHITE);
            let shown = galley.rows.iter().flat_map(|r| r.glyphs.iter().map(|g| g.chr)).collect();
            out = (shown, galley.size().x, dot.map_or(0.0, |d| d.size().x));
        });
        frame.textures_delta.clear();
        out
    }

    #[test]
    fn long_title_is_truncated_and_keeps_the_unsaved_marker() {
        let long = "a very long presentation name ".repeat(20);
        let (text, w, dot_w) = layout_title(&long, true, 200.0);
        assert!(text.ends_with('…'), "truncated title ends with an ellipsis: {text:?}");
        assert!(dot_w > 0.0, "the unsaved marker is laid out");
        assert!(w + dot_w <= 200.0, "title and marker fit together: {}", w + dot_w);
        let (text, _, dot_w) = layout_title("Northern Lights", false, 300.0);
        assert_eq!(text, "Northern Lights");
        assert_eq!(dot_w, 0.0);
    }
}

#[cfg(test)]
mod drawtable_tests;

#[cfg(test)]
mod inktoshape_tests;

#[cfg(test)]
mod picture_styles_tests;
