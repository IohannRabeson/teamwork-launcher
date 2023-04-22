use std::error::Error;
///! See https://docs.rs/vergen/latest/vergen/ for more info
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // Emit the instructions
    EmitBuilder::builder().git_sha(true).emit()?;
    Ok(())
}
