use super::{Snapshot, files};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

const POLL: Duration = Duration::from_secs(1);

pub(super) struct Worker {
    latest: Arc<Mutex<Snapshot>>,
    stop: mpsc::SyncSender<()>,
}

impl Worker {
    pub fn start(ctx: egui::Context) -> Option<Self> {
        let roots = files::roots(
            std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
            std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            std::env::var_os("HOME").map(PathBuf::from),
        );
        let latest = Arc::new(Mutex::new(Snapshot::default()));
        let shared = latest.clone();
        let (stop, stopped) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("omadeck-appearance".into())
            .spawn(move || {
                let mut selected = None;
                let mut last = Snapshot::default();
                let mut font_poll = 0_u8;
                loop {
                    let mut next = last.clone();
                    if let Some(palette) = files::read_palette(&roots, &mut selected) {
                        next.palette = Some(palette);
                    }
                    if font_poll == 0 || next.palette != last.palette {
                        // The system UI stack (exact family matches only) and the user's fontconfig
                        // monospace. Chrome text stays Inter; mono is only for code surfaces.
                        next.fonts = Some(omacraft_ui::SystemFaces::detect());
                    }
                    font_poll = (font_poll + 1) % 10;
                    if next != last {
                        if let Ok(mut value) = shared.lock() {
                            *value = next.clone();
                        }
                        last = next;
                        ctx.request_repaint();
                    }
                    if !matches!(stopped.recv_timeout(POLL), Err(mpsc::RecvTimeoutError::Timeout)) {
                        break;
                    }
                }
            })
            .ok()?;
        Some(Self { latest, stop })
    }

    pub fn current(&self) -> Option<Snapshot> {
        self.latest.try_lock().ok().map(|value| value.clone())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.stop.try_send(());
    }
}
