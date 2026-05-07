use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    about = "Clipboard image bridge for terminal AI agents",
    long_about = "Watches the clipboard for images. When one is detected, saves it to ~/.clipbridge/cache/ and \
                  replaces the clipboard with a text payload pointing at the saved file. Pasting that text into \
                  Claude Code / Gemini CLI / Codex makes them open the file with their image-reading tool."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the watcher in the foreground (default if no subcommand). Ctrl+C to stop.
    Start,

    /// Run a command with the watcher active for its lifetime.
    /// Example: clipbridge run -- claude
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1..)]
        args: Vec<String>,
    },

    /// Verify the environment (clipboard access, cache directory).
    Doctor,
}
