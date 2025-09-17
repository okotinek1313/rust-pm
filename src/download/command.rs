use std::fs::File;
use std::io::copy;
use reqwest::blocking::Client;
use std::env;
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub size: String,
    pub date: String,
    pub filename: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    pub url: String,
    pub architecture: String,
    pub package_count: usize,
    pub packages: Vec<Package>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageRegistry {
    pub repositories: Vec<Repository>,
    pub total_packages: usize,
}

pub enum DownloadTarget {
    PackageName(String),
    DirectUrl { url: String, filename: String },
}

pub fn exec(target: DownloadTarget) -> Result<String, Box<dyn std::error::Error>> {
    match target {
        DownloadTarget::PackageName(package_name) => {
            download_by_package_name(&package_name)
        },
        DownloadTarget::DirectUrl { url, filename } => {
            download_package(&url, &filename)
        }
    }
}

fn download_by_package_name(package_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Check if packages.json exists, if not, run parser
    if !fs::metadata("packages.json").is_ok() {
        println!("Package index not found, building index...");
        crate::parser::command::exec(None)?;
    }
    
    // Load package registry
    let registry = load_package_registry()?;
    
    // Search for the package
    let found_packages = search_packages(&registry, package_name);
    
    if found_packages.is_empty() {
        return Err(format!("Package '{}' not found. Try running 'rust-pm parse' to refresh the package index.", package_name).into());
    }
    
    // If multiple matches, show them and pick the first one
    if found_packages.len() > 1 {
        println!("Multiple packages found:");
        for (i, (repo, package)) in found_packages.iter().enumerate() {
            println!("  {}: {} ({}) from {}", i + 1, package.name, package.version, repo.url);
        }
        println!("Using: {} ({})", found_packages[0].1.name, found_packages[0].1.version);
    }
    
    let (repository, package) = &found_packages[0];
    
    // Construct download URL
    let download_url = format!("{}{}", repository.url, package.filename);
    
    println!("🎯 Found package: {} ({})", package.name, package.version);
    
    download_package(&download_url, &package.filename)
}

fn search_packages<'a>(registry: &'a PackageRegistry, search_term: &str) -> Vec<(&'a Repository, &'a Package)> {
    let mut results = Vec::new();
    let search_lower = search_term.to_lowercase();
    
    for repository in &registry.repositories {
        for package in &repository.packages {
            // Exact match first
            if package.name.to_lowercase() == search_lower {
                results.insert(0, (repository, package)); // Insert at beginning for exact matches
            }
            // Partial match
            else if package.name.to_lowercase().contains(&search_lower) {
                results.push((repository, package));
            }
        }
    }
    
    results
}

fn load_package_registry() -> Result<PackageRegistry, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("packages.json")?;
    let registry: PackageRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

fn download_package(url: &str, package_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Get home directory and create downloads path
    let home_dir = env::var("HOME")?;
    let downloads_dir = format!("{}/code/rust-pm/downloads", home_dir);
    
    // Create downloads directory if it doesn't exist
    std::fs::create_dir_all(&downloads_dir)?;
    
    // Create output path
    let output_path = format!("{}/{}", downloads_dir, package_name);
    
    println!("Downloading: {}", url);
    println!("Saving to: {}", output_path);
    
    // Create HTTP client and download
    let client = Client::new();
    let mut response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }
    
    let mut dest = File::create(&output_path)?;
    copy(&mut response, &mut dest)?;
    
    println!("Download completed successfully");
    Ok(output_path)
}
