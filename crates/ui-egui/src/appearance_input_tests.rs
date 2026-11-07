//! Live desktop font changes must wait for an editor to finish, including its physical input.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{Services, SlideApp, appearance, omarchy, theme};
use deckcraft_engine::Session;
use egui::{Event, FontFamily, ImeEvent};
use serde_json::json;

fn harness() -> egui_kittest::Harness<'static, SlideApp> {
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        SlideApp::new(Session::with_new(), Services::default()),
    );
    crate::install_ime_pass_hooks(&h.ctx);
    h.run_steps(3);
    h.state_mut().ui.follow_desktop = true;
    let title = h.state().session.active().unwrap().doc.slides[0].shapes[0].id.0;
    h.state_mut().run("text.edit", json!({"id": title})).unwrap();
    h.run_steps(2);
    h
}

fn title_text(app: &SlideApp) -> String {
    app.session.active().unwrap().doc.slides[0].shapes[0].text.as_ref().unwrap().text()
}

/// A system UI face arrives from the worker while "Use system font" is on.
fn queue_desktop_font(h: &mut egui_kittest::Harness<'static, SlideApp>) {
    let face = omacraft_ui::fonts::SystemFace {
        family: "Test Sans".into(),
        path: "/test/TestSans-Regular.ttf".into(),
        index: 0,
        data: std::sync::Arc::new(omacraft_ui::fonts::INTER_REGULAR.to_vec()),
    };
    let palette = omarchy::Palette::parse("background='#101020'\nforeground='#eeeeff'\naccent='#bbaaee'", false).unwrap();
    let fonts = omacraft_ui::SystemFaces { ui: vec![face], ..Default::default() };
    h.state_mut().ui.use_system_font = true;
    h.state_mut().appearance = omarchy::Watcher::fixed_snapshot(omarchy::Snapshot { palette: Some(palette), fonts: Some(fonts) });
}

fn desktop_font_installed(ctx: &egui::Context) -> bool {
    let mut definitions = theme::font_definitions(&[]);
    appearance::fonts(ctx, &mut definitions);
    definitions.families[&FontFamily::Proportional].first().is_some_and(|name| name.contains("TestSans"))
}

fn leave_editor_and_apply(h: &mut egui_kittest::Harness<'static, SlideApp>) {
    h.state_mut().run("text.exit", json!({})).unwrap();
    h.ctx.memory_mut(|memory| {
        if let Some(id) = memory.focused() {
            memory.surrender_focus(id);
        }
    });
    h.run_steps(3);
    assert!(desktop_font_installed(&h.ctx), "latest font snapshot applies once editing has ended");
}

#[test]
fn desktop_font_change_preserves_physical_text_and_waits_for_editor_exit() {
    let mut h = harness();
    queue_desktop_font(&mut h);
    h.input_mut().events.push(Event::Text("hello".into()));
    h.run_steps(1);
    assert_eq!(title_text(h.state()), "hello");
    assert!(h.state().styled && h.state().fonts_ready);
    assert!(!desktop_font_installed(&h.ctx), "no font-install/discard while the editor owns input");
    h.run_steps(1);
    assert_eq!(title_text(h.state()), "hello", "physical input is neither lost nor replayed twice");
    leave_editor_and_apply(&mut h);
    assert_eq!(title_text(h.state()), "hello");
}

#[test]
fn desktop_font_change_preserves_ime_preedit_commit_and_single_undo() {
    let mut h = harness();
    let undo = h.state().session.active().unwrap().history.undo.len();
    h.event(Event::Ime(ImeEvent::Preedit { text: "日".into(), active_range_chars: None }));
    h.run_steps(1);
    assert_eq!(title_text(h.state()), "日");
    queue_desktop_font(&mut h);
    h.event(Event::Ime(ImeEvent::Preedit { text: "日本".into(), active_range_chars: None }));
    h.run_steps(1);
    assert_eq!(title_text(h.state()), "日本", "pending font change must not retire the active composition");
    assert!(!desktop_font_installed(&h.ctx));
    h.event(Event::Ime(ImeEvent::Commit("日本".into())));
    h.run_steps(2);
    assert_eq!(title_text(h.state()), "日本");
    assert_eq!(h.state().session.active().unwrap().history.undo.len(), undo + 1);
    assert!(h.state().session.active().unwrap().composition.is_none());
    leave_editor_and_apply(&mut h);
    h.state_mut().run("edit.undo", json!({})).unwrap();
    h.run_steps(1);
    assert_eq!(title_text(h.state()), "", "one undo still removes the committed composition");
}
