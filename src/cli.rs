use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    about = "Clip-Claude (即貼即克) — clipboard image bridge for terminal AI agents",
    long_about = "Watches the clipboard for images. When a screenshot arrives, saves it as a PNG \
                  and augments the clipboard so it carries BOTH the original image (CF_DIB matching \
                  what .NET / Snipping Tool produce, byte-for-byte) and a text path payload \
                  (CF_UNICODETEXT). Image-paste apps take the image; terminal agents (Claude Code, \
                  Gemini CLI, Codex) take the text path and open the file via their Read tool. \
                  Run `clip-claude install` once and it auto-starts at every login, invisible."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the watcher in the foreground (default). Ctrl+C to stop.
    Start,

    /// Run a command with the watcher active for its lifetime.
    /// Example: clip-claude run -- claude
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1..)]
        args: Vec<String>,
    },

    /// Verify environment (clipboard access, cache directory).
    Doctor,

    /// Install: copy binaries to %LOCALAPPDATA%\Clip-Claude\, register an HKCU Run entry
    /// so the invisible background daemon launches at every login, and start it now.
    Install,

    /// Uninstall: stop the running daemon and remove the auto-start registry entry.
    /// Leaves the binaries behind; remove them manually if you want.
    Uninstall,

    /// Show whether Clip-Claude is installed, the registry entry is present, and the daemon is running.
    Status,
}
