use std::fs::File;
use std::io::copy;
use reqwest::blocking::Client;
use std::env;

pub fn download_package(url: &str, package_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Get home directory and create downloads path
    let home_dir = env::var("HOME")?;
    let downloads_dir = format!("{}/code/rust-pm/downloads", home_dir);
    
    // Create downloads directory if it doesn't exist
    std::fs::create_dir_all(&downloads_dir)?;
    
    // Create output path
    let output_path = format!("{}/{}", downloads_dir, package_name);
    
    println!("📦 Downloading: {}", url);
    println!("💾 Saving to: {}", output_path);
    
    // Create HTTP client and download
    let client = Client::new();
    let mut response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }
    
    let mut dest = File::create(&output_path)?;
    copy(&mut response, &mut dest)?;
    
    println!("✅ Download completed!");
    Ok(output_path)
}

pub fn exec(url: &str, package_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    download_package(url, package_name)
}
