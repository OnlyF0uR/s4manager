use clap::{Parser, Subcommand};
use config::{load_config, Config};
use inquire::Text;

pub mod compile;
pub mod config;
pub mod decompile;
pub mod errors;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile { input: String },
    Decompile { directory: Option<String> },
    Configure,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input } => {
            if Config::exists() {
                if let Err(e) = load_config().await {
                    eprintln!("Failed to load config: {}", e);
                    return;
                };
            } else {
                println!("Config file does not exist. Please run `s4m configure` to create one.");
                return;
            }

            match compile::execute(input).await {
                Ok(_) => println!("Compilation successful."),
                Err(e) => eprintln!("Compilation failed: {}", e),
            };
        }
        Commands::Decompile { directory } => {
            if Config::exists() {
                if let Err(e) = load_config().await {
                    eprintln!("Failed to load config: {}", e);
                    return;
                };
            } else {
                println!("Config file does not exist. Please run `s4m configure` to create one.");
                return;
            }

            match decompile::execute(directory.as_ref().map(AsRef::as_ref)).await {
                Ok(_) => println!("Decompilation successful."),
                Err(e) => eprintln!("Decompilation failed: {}", e),
            }
        }
        Commands::Configure => {
            // This will ask the user questions for the config file
            let author = match Text::new("What is the author's name?").prompt() {
                Ok(author) => author,
                Err(e) => {
                    eprintln!("Failed to get author name: {}", e);
                    return;
                }
            };

            let py_path = match Text::new("What is the path to the Python root directory?").prompt()
            {
                Ok(py_path) => py_path,
                Err(e) => {
                    eprintln!("Failed to get Python path: {}", e);
                    return;
                }
            };

            let s4_mods_path =
                match Text::new("What is the path to the Sims 4 mods directory?").prompt() {
                    Ok(s4_mods_path) => s4_mods_path,
                    Err(e) => {
                        eprintln!("Failed to get Sims 4 mods path: {}", e);
                        return;
                    }
                };

            let s4_install_path =
                match Text::new("What is the path to the Sims 4 installation?").prompt() {
                    Ok(s4_install_path) => s4_install_path,
                    Err(e) => {
                        eprintln!("Failed to get Sims 4 installation path: {}", e);
                        return;
                    }
                };

            if let Err(e) =
                config::write_config(&author, &py_path, &s4_mods_path, &s4_install_path).await
            {
                eprintln!("Failed to write config: {}", e);
                return;
            }
        }
    }
}
