use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    about = "Clipboard image bridge for terminal AI agents",
    long_about = "Watches the clipboard for images. By default, only converts when an agent CLI \
                  (claude / gemini / codex) is detected in the foreground window's process tree — \
                  so screenshots pasted into Photoshop / chat apps / etc. behave normally. \
                  Pasting into an agent terminal yields a text payload pointing at a saved PNG, \
                  which the agent's Read tool opens multimodally."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the watcher in the foreground (default if no subcommand). Ctrl+C to stop.
    Start {
        /// Convert every image-only clipboard regardless of foreground window.
        /// Without this flag (default), only converts when an agent CLI is detected in the
        /// foreground process tree.
        #[arg(long)]
        all_windows: bool,
    },

    /// Run a command with the watcher active for its lifetime.
    /// Focus check still applies: if you Alt-Tab away from the wrapped agent, conversion pauses.
    /// Example: clipbridge run -- claude
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1..)]
        args: Vec<String>,
    },

    /// Verify environment (clipboard access, cache directory, current foreground detection).
    Doctor,
}
