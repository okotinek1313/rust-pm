mod download;
mod init;
mod install;
mod extract;
mod uninstall;
mod parser;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Install,
    Uninstall,
    Extract,
    Download { url: String, name: String },
    Parse { 
        #[arg(short, long)]
        url: Option<String>
    }
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init => {
            if let Err(e) = init::command::exec() {
                eprintln!("Error initializing: {}", e);
            }
        },
        Commands::Install => {
            println!("Install command not yet implemented");
        },
        Commands::Uninstall => {
            println!("Uninstall command not yet implemented");
        },
        Commands::Extract => {
            println!("Extract command not yet implemented");
        },
        Commands::Download { url, name } => {
            if let Err(e) = download::command::exec(&url, &name) {
                eprintln!("Error downloading: {}", e);
            }
        },
        Commands::Parse { url } => {
            if let Err(e) = parser::command::exec(url) {
                eprintln!("Error parsing packages: {}", e);
            }
        }
    }
}
