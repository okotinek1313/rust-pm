use std::env;
use std::fs;

// Public struct - other modules can use this
pub struct PathConfig {
    pub path_dir: String,
    pub path_usr_dir: String,
    pub path_bin_dir: String,
    pub path_lib_dir: String,
    pub path_share_dir: String,
}

// Private function - only accessible within this module
fn create_paths() -> Result<PathConfig, Box<dyn std::error::Error>> {
    let home_dir = env::var("HOME")?;
    
    let path_dir = format!("{}/.path", home_dir);
    let path_usr_dir = format!("{}/.path/usr", home_dir);
    let path_bin_dir = format!("{}/.path/usr/bin", home_dir);
    let path_lib_dir = format!("{}/.path/usr/lib", home_dir);
    let path_share_dir = format!("{}/.path/usr/share", home_dir);

    // Create all directories
    fs::create_dir_all(&path_bin_dir)?;
    fs::create_dir_all(&path_lib_dir)?;
    fs::create_dir_all(&path_share_dir)?;

    println!("Created all package directories");

    Ok(PathConfig {
        path_dir,
        path_usr_dir,
        path_bin_dir,
        path_lib_dir,
        path_share_dir,
    })
}

// Public execution function - main entry point
pub fn exec() -> Result<PathConfig, Box<dyn std::error::Error>> {
    println!("Initializing package directories...");
    create_paths()
}
