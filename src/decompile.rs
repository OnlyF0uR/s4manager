use crate::errors::DecompileError;

pub async fn execute(directory: Option<&str>) -> Result<(), DecompileError> {
    println!(
        "Decompiling into: {:?}",
        directory.unwrap_or("current directory")
    );

    // TODO: This

    Ok(())
}
