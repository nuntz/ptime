pub mod analysis;
pub mod cli;
pub mod error;
pub mod metadata;
pub mod render;
pub mod scanner;

use cli::{Cli, CommandKind};

pub fn run() -> anyhow::Result<()> {
    let cmd = Cli::parse_args().map_err(|e| anyhow::anyhow!("CLI parsing error: {}", e))?;

    let photos = match metadata::collect_photos(&cmd.directory) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(e.exit_code());
        }
    };

    match cmd.kind {
        CommandKind::Oldest => {
            if let Some(photo) = analysis::find_oldest(&photos) {
                println!("{} {}", photo.rel_path.display(), photo.date);
            }
            // Empty output for no photos
        }
        CommandKind::Latest => {
            if let Some(photo) = analysis::find_latest(&photos) {
                println!("{} {}", photo.rel_path.display(), photo.date);
            }
            // Empty output for no photos
        }
        CommandKind::Hist { width } => {
            let histogram = analysis::build_histogram(&photos);
            let lines = render::render_histogram(&histogram, width);
            for line in lines {
                println!("{}", line);
            }
            // Empty output for no photos
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // The run() function now requires CLI args, so we can't test it in isolation.
    // Integration tests will cover the full CLI flow.
}
