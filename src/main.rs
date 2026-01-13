use anyhow::{Context, Error, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Configuration file schema.
#[derive(Debug, Deserialize, Serialize, Default)]
struct Config {
    apps: Vec<AppConfig>,
}

/// Configuration for a single application.
#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    name: String,
    path: PathBuf,
    light_token: String,
    dark_token: String,
    reload_cmd: Option<String>,
}

/// Command line interface for synchronizing theme choices across tools.
#[derive(Parser)]
#[command(author, version, about = "Synchronize theme choices across tools", long_about = None)]
struct Cli {
    /// Override the configuration file path.
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: CommandKind,
}

/// Subcommands exposed by the CLI.
#[derive(Subcommand)]
enum CommandKind {
    /// Watch GNOME theme preference changes and apply them live.
    Monitor,
    /// Apply a theme once, optionally overriding the detected preference.
    Set {
        /// Explicit theme to apply instead of reading gsettings.
        #[arg(long, value_enum)]
        theme: Option<ThemePreference>,
    },
}

/// Normalized theme preference tokens reported by GNOME.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum ThemePreference {
    Dark,
    Light,
}

/// Applies theme updates for a single application defined in the configuration.
struct Configurator<'a> {
    app: &'a AppConfig,
}

impl<'a> Configurator<'a> {
    fn new(app: &'a AppConfig) -> Self {
        Self { app }
    }

    fn apply(&self, theme: ThemePreference) -> Result<()> {
        let (from, to, variant) = match theme {
            ThemePreference::Dark => (
                self.app.light_token.as_str(),
                self.app.dark_token.as_str(),
                "dark",
            ),
            ThemePreference::Light => (
                self.app.dark_token.as_str(),
                self.app.light_token.as_str(),
                "light",
            ),
        };

        info!("Applying {} theme to {}", variant, self.app.name);

        let home = std::env::var("SNAP_REAL_HOME")
            .or_else(|_| std::env::var("HOME"))
            .context("SNAP_REAL_HOME or HOME environment variable not set")?;

        let path = Path::new(&home).join(&self.app.path);

        replace_in_file(&path, from, to)
            .with_context(|| format!("updating {} theme", self.app.name))?;

        if let Some(reload) = self.app.reload_cmd.as_deref()
            && let Err(_) = run_command(reload)
        {
            warn!("Failed to reload {}", self.app.name);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let config = load_config(cli.config)?;

    match cli.command {
        CommandKind::Monitor => monitor_theme_changes(&config),
        CommandKind::Set { theme } => set_once(theme, &config),
    }
}

fn load_config(override_path: Option<PathBuf>) -> Result<Config, Error> {
    if let Some(path) = override_path {
        info!("Loading configuration from {}", path.display());
        confy::load_path::<Config>(&path).context("loading configuration from override path")
    } else {
        // Confy's default path loading doesn't play well with snaps, so we emulate it here
        // to account for both in-snap and out of snap invocations.
        let home = std::env::var("SNAP_REAL_HOME")
            .or_else(|_| std::env::var("HOME"))
            .context("SNAP_REAL_HOME or HOME environment variable not set")?;
        let path = Path::new(&home).join(".config/theme-sync/default-config.yml");

        info!("Loading configuration from {}", path.display());
        confy::load_path::<Config>(&path).context("loading configuration from default path")
    }
}

/// Stream GNOME theme preference updates and apply them to each tool.
fn monitor_theme_changes(config: &Config) -> Result<()> {
    let mut child = Command::new("gsettings")
        .args(["monitor", "org.gnome.desktop.interface", "color-scheme"])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .context("failed to capture gsettings output")?;

    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line?;
        let theme = infer_theme(&line);
        apply_all(theme, config)?;
    }

    child.wait()?;
    Ok(())
}

/// Snapshot the current GNOME theme preference (or override) and apply it once.
fn set_once(theme_override: Option<ThemePreference>, config: &Config) -> Result<()> {
    let theme = match theme_override {
        Some(theme) => theme,
        None => {
            let output = Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "color-scheme"])
                .output()?;

            if !output.status.success() {
                bail!("gsettings get failed");
            }

            let stdout = String::from_utf8(output.stdout)?;
            infer_theme(&stdout)
        }
    };

    apply_all(theme, config)
}

/// Run every configurator for the supplied preference.
fn apply_all(theme: ThemePreference, config: &Config) -> Result<()> {
    for app in &config.apps {
        Configurator::new(app).apply(theme)?;
    }
    Ok(())
}

/// Infer a normalized theme choice from the gsettings output.
fn infer_theme(input: &str) -> ThemePreference {
    if input.contains("prefer-dark") {
        ThemePreference::Dark
    } else {
        ThemePreference::Light
    }
}

/// Replace occurrences of `from` with `to` in the provided file if needed.
fn replace_in_file(path: &Path, from: &str, to: &str) -> Result<()> {
    let contents = fs::read_to_string(path)?;
    let replaced = contents.replace(from, to);
    if replaced != contents {
        debug!("Replacing `{}` with `{}` in {}", from, to, path.display());
        fs::write(path, replaced)?;
    }
    Ok(())
}

/// Execute a shell command via `bash -c` and surface failures as anyhow errors.
fn run_command(command: &str) -> Result<()> {
    debug!("Running command: {}", command);
    let status = Command::new("bash").args(["-c", command]).status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("command `{command}` exited with status {status}")
    }
}
