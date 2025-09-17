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
    Download {
        /// Package name to download, or URL if --url flag is used
        package: String,
        /// Download from direct URL (requires filename as package argument)
        #[arg(long)]
        url: bool,
    },
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
        Commands::Download { package, url } => {
            use download::command::DownloadTarget;
            
            let target = if url {
                // If --url flag is used, treat package as filename and first arg as URL
                // This is a bit hacky, but works for the current interface
                // Better would be to have separate url and filename args
                DownloadTarget::DirectUrl { 
                    url: package.clone(), 
                    filename: package.split('/').last().unwrap_or(&package).to_string()
                }
            } else {
                DownloadTarget::PackageName(package)
            };
            
            if let Err(e) = download::command::exec(target) {
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
