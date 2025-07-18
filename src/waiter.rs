use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Waiter {
    spinner: ProgressBar,
}

impl Waiter {
    pub(crate) fn start() -> Self {
        let style = ProgressStyle::with_template("{msg}   {spinner}")
            .expect("Invalid template")
            .tick_strings(&["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"]);
        let spinner = ProgressBar::new_spinner()
            .with_style(style)
            .with_message("Waiting");
        spinner.enable_steady_tick(Duration::from_millis(50));
        Waiter { spinner }
    }

    pub(crate) fn stop(self) {

        self.spinner.finish_and_clear();
        drop(self.spinner);
    }
}

