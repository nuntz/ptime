pub mod analysis;
pub mod cli;
pub mod error;
pub mod metadata;
pub mod render;
pub mod scanner;

pub fn run() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_returns_ok() {
        assert!(run().is_ok());
    }
}
