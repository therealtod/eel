//! Debug-only capture of the raw inbound websocket stream.
//!
//! When the bot plays on hanab.live, every server frame is the literal `label payload` text the
//! client parses with [`InboundWebsocketMessage::parse`](crate::hanablive::inbound_websocket_message::InboundWebsocketMessage::parse).
//! Appending each frame verbatim to a file produces a transcript that can be replayed offline,
//! one line at a time, through the exact same parsing/`BotState` code path the live bot used —
//! the faithful reproduction the full-deck `replay.json` reconstruction can't give (it lacks the
//! hidden own-hand draws and the server's clue `list`/turn ordering).
//!
//! The recorder is **compiled out of optimized (`--release`) builds**: in release,
//! [`LiveplayRecorder::new`] always returns `None` and the type carries no state, so there is no
//! file I/O and no runtime cost on the live hot path. In debug builds it is on by default; set
//! `EEL_LIVEPLAY_LOG_DIR` to choose the output directory (default `logs/`).

#[cfg(debug_assertions)]
pub use debug_impl::LiveplayRecorder;

#[cfg(not(debug_assertions))]
pub use release_stub::LiveplayRecorder;

#[cfg(debug_assertions)]
mod debug_impl {
    use std::fs::{self, File, OpenOptions};
    use std::io::{BufWriter, Write};
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// Appends each raw inbound websocket frame to a per-connection log file. Created once per
    /// successful connection; reconnects start a fresh file.
    pub struct LiveplayRecorder {
        writer: Mutex<BufWriter<File>>,
        path: PathBuf,
    }

    impl LiveplayRecorder {
        /// Open a new transcript file under `EEL_LIVEPLAY_LOG_DIR` (default `logs/`), named with a
        /// UTC timestamp. Returns `None` (and logs a warning) if the directory or file cannot be
        /// created, so recording failures never take down the bot.
        #[must_use]
        pub fn new() -> Option<Self> {
            let dir = std::env::var("EEL_LIVEPLAY_LOG_DIR").unwrap_or_else(|_| "logs".to_string());
            Self::new_in_dir(&dir)
        }

        fn new_in_dir(dir: &str) -> Option<Self> {
            if let Err(e) = fs::create_dir_all(dir) {
                tracing::warn!("Liveplay recorder: could not create dir '{dir}': {e}");
                return None;
            }
            let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
            let path = PathBuf::from(dir).join(format!("liveplay-{timestamp}.log"));
            let file = match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(file) => file,
                Err(e) => {
                    tracing::warn!(
                        "Liveplay recorder: could not open '{}': {e}",
                        path.display()
                    );
                    return None;
                }
            };
            tracing::info!(
                "Liveplay recorder: writing transcript to {}",
                path.display()
            );
            Some(Self {
                writer: Mutex::new(BufWriter::new(file)),
                path,
            })
        }

        /// Append one raw frame as a line. Flushes immediately so a transcript survives a crash or
        /// `kill` mid-game (the common case when debugging a misbehaving bot). I/O errors are
        /// logged once and otherwise ignored.
        pub fn record(&self, raw: &str) {
            let Ok(mut writer) = self.writer.lock() else {
                return;
            };
            if let Err(e) = writeln!(writer, "{raw}").and_then(|()| writer.flush()) {
                tracing::warn!(
                    "Liveplay recorder: write to {} failed: {e}",
                    self.path.display()
                );
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::LiveplayRecorder;

        #[test]
        fn records_each_frame_as_a_replayable_line() {
            // Write into a unique temp subdir so the test never collides with a real `logs/`.
            let dir =
                std::env::temp_dir().join(format!("eel-liveplay-test-{}", std::process::id()));
            let recorder =
                LiveplayRecorder::new_in_dir(dir.to_str().unwrap()).expect("recorder opens");

            let frames = [
                r#"gameActionList {"tableID":1,"list":[]}"#,
                r#"gameAction {"tableID":1,"action":{"type":"play","playerIndex":0,"order":0,"suitIndex":0,"rank":1}}"#,
            ];
            for f in frames {
                recorder.record(f);
            }

            let contents = std::fs::read_to_string(&recorder.path).expect("read transcript");
            let lines: Vec<&str> = contents.lines().collect();
            assert_eq!(
                lines, frames,
                "each frame must be one verbatim, replayable line"
            );

            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

#[cfg(not(debug_assertions))]
mod release_stub {
    /// Zero-sized no-op recorder used in optimized builds: never opens a file and compiles the
    /// `record` call down to nothing.
    pub struct LiveplayRecorder;

    impl LiveplayRecorder {
        #[must_use]
        pub fn new() -> Option<Self> {
            None
        }

        #[inline]
        pub fn record(&self, _raw: &str) {}
    }
}
