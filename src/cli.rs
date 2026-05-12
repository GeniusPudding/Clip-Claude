use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    about = "Clipboard image bridge for terminal AI agents",
    long_about = "Watches the clipboard for images. When a screenshot arrives, saves it as a PNG \
                  and augments the clipboard so it carries BOTH the original image (CF_DIB) and a \
                  text payload pointing at the saved file (CF_UNICODETEXT). Image-paste apps \
                  (Photoshop, Telegram, etc.) take the image; terminal agents (Claude Code, \
                  Gemini CLI, Codex) take the text payload and open the file via their Read tool."
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

    /// Verify environment (clipboard access, cache directory).
    Doctor,
}
