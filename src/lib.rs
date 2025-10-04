pub mod analysis;
pub mod cli;
pub mod error;
pub mod metadata;
pub mod render;
pub mod scanner;

pub fn run() -> anyhow::Result<()> {
    let _cmd = cli::Cli::parse_args().map_err(|e| anyhow::anyhow!("CLI parsing error: {}", e))?;
    // Command execution will be implemented later
    Ok(())
}

#[cfg(test)]
mod tests {
    // The run() function now requires CLI args, so we can't test it in isolation.
    // Integration tests will cover the full CLI flow.
}
