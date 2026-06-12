use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    println!("KeyFlow v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}