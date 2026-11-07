use super::{Dialog, SlideApp, Value, json, theme};

/// What the Preferences dialog can change, as it was when the dialog opened.
pub(super) fn preferences_snapshot(app: &SlideApp) -> Value {
    json!({
        "followDesktop": app.ui.follow_desktop,
        "useSystemFont": app.ui.use_system_font,
        "dark": app.ui.brightness == theme::Brightness::Dark,
        "uiScale": app.ui.ui_scale,
        "prefs": serde_json::to_value(&app.session.prefs).unwrap_or(Value::Null),
    })
}

/// Puts back the settings the Preferences dialog opened with (Cancel, Esc, the close button).
pub(super) fn restore_preferences(app: &mut SlideApp, d: &Dialog) {
    let Some(before) = d.params.get("before") else { return };
    if let Some(dark) = before.get("dark").and_then(Value::as_bool)
        && dark != (app.ui.brightness == theme::Brightness::Dark)
    {
        let _ = app.run("view.dark", json!({ "on": dark }));
    }
    if let Some(follow) = before.get("followDesktop").and_then(Value::as_bool) {
        let _ = app.run("view.systemTheme", json!({"on": follow}));
    }
    if let Some(system_font) = before.get("useSystemFont").and_then(Value::as_bool) {
        app.ui.use_system_font = system_font;
    }
    if let Some(scale) = before.get("uiScale").and_then(Value::as_f64) {
        app.ui.ui_scale = scale as f32;
    }
    if let Some(prefs) = before.get("prefs").and_then(|p| serde_json::from_value(p.clone()).ok()) {
        app.session.prefs = prefs;
    }
}
