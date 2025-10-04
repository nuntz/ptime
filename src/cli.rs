use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ptime")]
#[command(about = "Analyze photo timestamps from JPEG files")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Find the oldest photo
    Oldest {
        /// Directory to scan (default: current directory)
        directory: Option<PathBuf>,
    },
    /// Find the most recent photo
    Latest {
        /// Directory to scan (default: current directory)
        directory: Option<PathBuf>,
    },
    /// Show histogram of photos by year
    Hist {
        /// Width of histogram bars (1-200, clamped at 200)
        #[arg(short, long, default_value = "50")]
        width: usize,
        /// Directory to scan (default: current directory)
        directory: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub struct CliCommand {
    pub kind: CommandKind,
    pub directory: PathBuf,
}

#[derive(Debug)]
pub enum CommandKind {
    Oldest,
    Latest,
    Hist { width: usize },
}

impl Cli {
    pub fn parse_args() -> Result<CliCommand, String> {
        let cli = Cli::parse();
        Self::convert(cli)
    }

    fn convert(cli: Cli) -> Result<CliCommand, String> {
        let (kind, directory) = match cli.command {
            Command::Oldest { directory } => {
                let dir = directory.unwrap_or_else(|| PathBuf::from("."));
                (CommandKind::Oldest, dir)
            }
            Command::Latest { directory } => {
                let dir = directory.unwrap_or_else(|| PathBuf::from("."));
                (CommandKind::Latest, dir)
            }
            Command::Hist { width, directory } => {
                if width == 0 {
                    return Err("Width must be at least 1".to_string());
                }
                let clamped_width = width.min(200);
                let dir = directory.unwrap_or_else(|| PathBuf::from("."));
                (
                    CommandKind::Hist {
                        width: clamped_width,
                    },
                    dir,
                )
            }
        };

        Ok(CliCommand { kind, directory })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oldest_with_default_dir() {
        let cli = Cli {
            command: Command::Oldest { directory: None },
        };
        let result = Cli::convert(cli).unwrap();
        assert!(matches!(result.kind, CommandKind::Oldest));
        assert_eq!(result.directory, PathBuf::from("."));
    }

    #[test]
    fn test_oldest_with_custom_dir() {
        let cli = Cli {
            command: Command::Oldest {
                directory: Some(PathBuf::from("/tmp/photos")),
            },
        };
        let result = Cli::convert(cli).unwrap();
        assert!(matches!(result.kind, CommandKind::Oldest));
        assert_eq!(result.directory, PathBuf::from("/tmp/photos"));
    }

    #[test]
    fn test_latest_with_default_dir() {
        let cli = Cli {
            command: Command::Latest { directory: None },
        };
        let result = Cli::convert(cli).unwrap();
        assert!(matches!(result.kind, CommandKind::Latest));
        assert_eq!(result.directory, PathBuf::from("."));
    }

    #[test]
    fn test_hist_with_default_width() {
        let cli = Cli {
            command: Command::Hist {
                width: 50,
                directory: None,
            },
        };
        let result = Cli::convert(cli).unwrap();
        match result.kind {
            CommandKind::Hist { width } => assert_eq!(width, 50),
            _ => panic!("Expected Hist command"),
        }
        assert_eq!(result.directory, PathBuf::from("."));
    }

    #[test]
    fn test_hist_with_custom_width() {
        let cli = Cli {
            command: Command::Hist {
                width: 100,
                directory: None,
            },
        };
        let result = Cli::convert(cli).unwrap();
        match result.kind {
            CommandKind::Hist { width } => assert_eq!(width, 100),
            _ => panic!("Expected Hist command"),
        }
    }

    #[test]
    fn test_hist_width_clamped_above_200() {
        let cli = Cli {
            command: Command::Hist {
                width: 300,
                directory: None,
            },
        };
        let result = Cli::convert(cli).unwrap();
        match result.kind {
            CommandKind::Hist { width } => assert_eq!(width, 200),
            _ => panic!("Expected Hist command"),
        }
    }

    #[test]
    fn test_hist_width_zero_errors() {
        let cli = Cli {
            command: Command::Hist {
                width: 0,
                directory: None,
            },
        };
        let result = Cli::convert(cli);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Width must be at least 1");
    }

    #[test]
    fn test_hist_with_custom_dir() {
        let cli = Cli {
            command: Command::Hist {
                width: 50,
                directory: Some(PathBuf::from("/tmp/pics")),
            },
        };
        let result = Cli::convert(cli).unwrap();
        assert_eq!(result.directory, PathBuf::from("/tmp/pics"));
    }
}
