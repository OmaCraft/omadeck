use super::{Align2, Color32, CornerRadius, Icon, Rect, Sense, SlideApp, Stroke, Tokens, Ui, icons, json, pos2, theme, vec2};

pub const TITLE_H: f32 = 38.0;

/// macOS draws the window into the title bar (`with_fullsize_content_view`) and AppKit paints the
/// traffic lights itself, 16 pt from the top of the window on macOS 27. Moving them would take
/// AppKit calls (`unsafe`, forbidden here), so on macOS the title bar is the band they are centred
/// in, with the controls beside them (#71). Up to macOS 15 the lights are smaller and centred 14 pt
/// from the top, about 2 pt above this bar's centre.
pub const MAC_TITLE_BAR: f32 = 32.0;

/// The traffic lights as AppKit lays them out, measured on macOS 27: button diameter,
/// centre-to-centre pitch and the first button's left edge.
pub(super) const MAC_LIGHT: f32 = 14.0;

pub(super) const MAC_LIGHT_PITCH: f32 = 23.0;

pub(super) const MAC_LIGHTS_LEFT: f32 = 9.0;

/// Where title-bar content starts on macOS: the end of the three lights plus the gap the lights
/// keep from the top of the window.
pub const MAC_CONTENT_LEFT: f32 = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT + (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;

/// The title bar's height, the left edge of its content and the height of its tallest control.
/// With the integrated title bar (macOS) the bar is shorter, so the controls shrink to keep a
/// margin above and below them.
pub(super) fn title_geometry(integrated: bool) -> (f32, f32, f32) {
    if integrated { (MAC_TITLE_BAR, MAC_CONTENT_LEFT, 22.0) } else { (TITLE_H, 10.0, 26.0) }
}

/// The window's title row: traffic-light space, quick access, centred title, actions.
pub fn title_bar(app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (height, left, control_h) = title_geometry(app.integrated_titlebar);
    egui::Panel::top("title_bar").exact_size(height).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let rect = ui.max_rect();
        // Dragging the empty title bar moves the window.
        let drag = ui.interact(rect, ui.scope_id().with("titledrag"), Sense::click_and_drag());
        // The drag surface is not a button: a full-width Button would be announced as one.
        drag.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Window, true, "Window title bar"));
        if drag.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            let max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
        }
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_max(pos2(rect.min.x + left, rect.min.y), pos2(rect.min.x + 420.0, rect.max.y)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.spacing_mut().item_spacing.x = 2.0;
        let has_doc = app.session.active().is_some();
        let dirty = app.session.active().is_some_and(|d| d.is_dirty());
        crate::qat::show(app, &mut child);
        let left_end = child.min_rect().right();
        let title = app.session.active().map(|d| d.title()).unwrap_or_else(|| "OmaDeck".into());
        // Right cluster, left to right: search box, Comments, Present, light/dark (spec "Title bar
        // right cluster"). Laid out right to left from the edge, over the bar's right half, so the
        // search box can take whatever room the centred title leaves.
        let mut right = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_max(pos2(rect.center().x, rect.min.y), pos2(rect.max.x - TITLE_RIGHT_PAD, rect.max.y)))
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        right.spacing_mut().item_spacing.x = 6.0;
        appearance_toggle(app, &mut right);
        // Deck's one primary mode action: outlined in the accent, never filled (spec exception).
        if present_button(&mut right, control_h, has_doc).clicked() && has_doc {
            let from = app.session.active().map(|d| d.selection.slide).unwrap_or(0);
            app.start_show(from, false);
        }
        if title_icon(&mut right, Icon::CommentsBubble, "Comments", has_doc, app.ui.pane.as_deref() == Some("comments"), control_h).clicked() {
            let _ = app.run("view.pane", json!({"pane": "comments", "toggle": true}));
        }
        // The search box ends at the shared actions slot whatever this app's actions are, so it
        // sits at the same place in every app; wider actions push it left.
        let next_right = right.min_rect().left() - right.spacing().item_spacing.x;
        let search_right = search_right_edge(rect.right(), next_right);
        right.add_space(next_right - search_right);
        // The room left of the slot once the whole title (and its marker) has its centred slot.
        let title_w = title_galleys(ui.painter(), &title, dirty, f32::INFINITY, t.text);
        let title_w = title_w.0.size().x + title_w.1.map_or(0.0, |d| d.size().x);
        let room = search_right - (rect.center().x + title_w / 2.0 + TITLE_GAP);
        if search_box(&mut right, search_width(room), control_h).clicked() {
            app.palette = Some((String::new(), 0));
        }
        // Centred title, cut with "…" so it never runs under the controls on either side.
        let max_w = title_max_width(rect.center().x, left_end, right.min_rect().left());
        paint_title(ui, rect.center(), &title, dirty, max_w, t.text);
    });
}

/// Gap between the window's right edge and the last title-bar action.
pub(super) const TITLE_RIGHT_PAD: f32 = 8.0;

/// Room reserved for the title-bar actions, measured from the window's right edge: the search
/// box's right edge sits here in every OmaCraft app (spec "Title bar right cluster").
pub(super) const TITLE_ACTIONS_SLOT: f32 = 300.0;

/// The search box's right edge in a bar ending at `bar_right` whose actions leave the next item
/// ending at `next_right`: the shared slot, or directly left of actions too wide for it.
pub(super) fn search_right_edge(bar_right: f32, next_right: f32) -> f32 {
    (bar_right - TITLE_ACTIONS_SLOT).min(next_right)
}

/// The light/dark toggle all OmaCraft apps share: click switches Dark Mode, right-click offers
/// System as well (it follows the desktop). Also in View › Dark Mode and Preferences.
fn appearance_toggle(app: &mut SlideApp, ui: &mut Ui) {
    let dark = app.ui.brightness == theme::Brightness::Dark;
    let resp = title_icon(ui, Icon::Appearance, "Dark Mode", true, false, APPEARANCE_H);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, dark, "Dark Mode"));
    if resp.clicked() {
        let _ = app.run("view.dark", json!({}));
    }
    resp.context_menu(|ui| {
        let system = app.ui.follow_desktop;
        for (label, on) in [("System", system), ("Light", !system && !dark), ("Dark", !system && dark)] {
            if ui.radio(on, label).clicked() {
                let _ = match label {
                    "System" => app.run("view.systemTheme", json!({"on": true})),
                    _ => app.run("view.dark", json!({"on": label == "Dark"})),
                };
                ui.close();
            }
        }
    });
}

/// The light/dark toggle's height: 28 × 26 in every app, also in the shorter macOS bar.
const APPEARANCE_H: f32 = 26.0;

/// Title-bar label sizes (docs/ui-consistency.md): the bold document title and the plain actions.
const TITLE_FONT: f32 = 13.0;
const ACTION_FONT: f32 = 12.5;

/// The search box: full width when there is room, shrinking to a minimum before the title has to
/// truncate, then an icon button (spec "Search box").
const SEARCH_W: f32 = 260.0;
const SEARCH_MIN_W: f32 = 140.0;
const TITLE_ICON_W: f32 = 28.0;

/// Width of the search box for `room` points beside the title; `TITLE_ICON_W` collapses it to an icon.
pub(super) fn search_width(room: f32) -> f32 {
    if room >= SEARCH_W {
        SEARCH_W
    } else if room >= SEARCH_MIN_W {
        room
    } else {
        TITLE_ICON_W
    }
}

/// The title-bar search box (or, at `TITLE_ICON_W`, its icon button). A button that opens the
/// command palette; ⇧⌘P still does the same.
fn search_box(ui: &mut Ui, width: f32, control_h: f32) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    const NAME: &str = "Search commands";
    if width <= TITLE_ICON_W {
        return title_icon(ui, Icon::Search, NAME, true, false, control_h).on_hover_text("Search commands (⇧⌘P)");
    }
    let (r, resp) = ui.allocate_exact_size(vec2(width, control_h), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, NAME));
    let fill = if resp.hovered() { t.field_bg } else { t.card };
    ui.painter().rect(r, CornerRadius::same(6), fill, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(16.0, 16.0)), Icon::Search, t.text_dim, false);
    let mut job = egui::text::LayoutJob::simple_singleline(NAME.to_string(), theme::font(ACTION_FONT), t.text_dim);
    job.wrap = egui::text::TextWrapping::truncate_at_width((width - 38.0).max(0.0));
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(pos2(r.min.x + 30.0, r.center().y - galley.size().y / 2.0), galley, t.text_dim);
    resp.on_hover_text("Search commands (⇧⌘P)")
}

/// A title-bar icon button, 28 wide and as tall as the bar's controls; `on` shows it selected.
fn title_icon(ui: &mut Ui, icon: Icon, tip: &str, enabled: bool, on: bool, control_h: f32) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(TITLE_ICON_W, control_h), if enabled { Sense::click() } else { Sense::hover() });
    let fill = if on {
        Some(t.selected)
    } else if enabled && resp.is_pointer_button_down_on() {
        Some(t.pressed)
    } else if enabled && resp.hovered() {
        Some(t.hover)
    } else {
        None
    };
    if let Some(c) = fill {
        ui.painter().rect_filled(r, CornerRadius::same(4), c);
    }
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(17.0, 17.0)), icon, t.text, !enabled);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, tip));
    resp.on_hover_text(tip)
}

/// Gap kept between the title and the controls beside it.
const TITLE_GAP: f32 = 12.0;

/// The widest the centred title may be without reaching the controls that end at `left_end` or
/// start at `right_start`.
pub(super) fn title_max_width(center_x: f32, left_end: f32, right_start: f32) -> f32 {
    let half = (center_x - left_end).min(right_start - center_x) - TITLE_GAP;
    (2.0 * half).max(0.0)
}

/// Lay out the title truncated to fit `max_w` together with the unsaved marker (when `dirty`).
pub(super) fn title_galleys(
    painter: &egui::Painter,
    title: &str,
    dirty: bool,
    max_w: f32,
    color: Color32,
) -> (std::sync::Arc<egui::Galley>, Option<std::sync::Arc<egui::Galley>>) {
    let font = theme::bold(TITLE_FONT);
    let dot = dirty.then(|| painter.layout_no_wrap(" •".to_string(), font.clone(), color));
    let dot_w = dot.as_ref().map_or(0.0, |g| g.size().x);
    let mut job = egui::text::LayoutJob::simple_singleline(title.to_string(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width((max_w - dot_w).max(0.0));
    (painter.layout_job(job), dot)
}

/// Paint the title centred on `center`, truncated with "…" to `max_w`. The unsaved marker stays
/// visible however short the title gets.
fn paint_title(ui: &Ui, center: egui::Pos2, title: &str, dirty: bool, max_w: f32, color: Color32) {
    let (galley, dot) = title_galleys(ui.painter(), title, dirty, max_w, color);
    let dot_w = dot.as_ref().map_or(0.0, |g| g.size().x);
    let galley_w = galley.size().x;
    let left = center.x - (galley_w + dot_w) / 2.0;
    let top = center.y - galley.size().y / 2.0;
    ui.painter().galley(pos2(left, top), galley, color);
    if let Some(dot) = dot {
        ui.painter().galley(pos2(left + galley_w, top), dot, color);
    }
}

/// The Present button: accent outline and accent text, the hover/pressed fill of every other
/// title-bar action, no solid accent fill.
fn present_button(ui: &mut Ui, control_h: f32, enabled: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let font = theme::font(ACTION_FONT);
    let tw = ui.fonts_mut(|f| f.layout_no_wrap("Present".to_string(), font.clone(), t.accent).size().x);
    let (rect, resp) = ui.allocate_exact_size(vec2(tw + 24.0, control_h - 2.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, "Present"));
    let radius = CornerRadius::same(4);
    if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(rect, radius, t.pressed);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, radius, t.hover);
    }
    let ink = if enabled { t.accent } else { t.text_faint };
    ui.painter().rect_stroke(rect, radius, Stroke::new(1.0, ink), egui::StrokeKind::Inside);
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Present", font, ink);
    resp.on_hover_text("Present from the current slide")
}
