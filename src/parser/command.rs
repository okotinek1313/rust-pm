use std::fs::File;
use std::io::Write;
use serde::{Serialize, Deserialize};
use scraper::{Html, Selector};
use reqwest::blocking::Client;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServersConfig {
    pub repositories: Vec<String>,
}

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

pub fn exec(url: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let urls = if let Some(single_url) = url {
        // Single URL provided via command line
        vec![single_url]
    } else {
        // Load URLs from servers.json
        load_servers_config()?.repositories
    };

    // Load existing registry or create new one
    let mut registry = load_existing_registry().unwrap_or_else(|_| PackageRegistry {
        repositories: Vec::new(),
        total_packages: 0,
    });
    
    let client = Client::new();
    
    for target_url in urls {
        println!("🔍 Parsing Alpine Linux packages from: {}", target_url);
        
        // Fetch the HTML content
        let response = client.get(&target_url).send()?;
        
        if !response.status().is_success() {
            eprintln!("⚠️ Failed to fetch repository page {}: {}", target_url, response.status());
            continue;
        }
        
        let html_content = response.text()?;
        let document = Html::parse_document(&html_content);
        
        // Parse packages from the HTML
        let packages = parse_packages(&document)?;
        
        println!("📦 Found {} packages", packages.len());
        
        // Check if repository already exists and update it, or add new one
        let new_repo = Repository {
            url: target_url.clone(),
            architecture: "x86_64".to_string(),
            package_count: packages.len(),
            packages,
        };
        
        let mut found_existing = false;
        for repo in &mut registry.repositories {
            if repo.url == target_url {
                println!("🔄 Updating existing repository: {}", target_url);
                *repo = new_repo.clone();
                found_existing = true;
                break;
            }
        }
        
        if !found_existing {
            println!("➕ Adding new repository: {}", target_url);
            registry.repositories.push(new_repo);
        }
    }
    
    // Update total package count
    registry.total_packages = registry.repositories.iter().map(|r| r.package_count).sum();
    
    // Write to JSON file
    let json_output = serde_json::to_string_pretty(&registry)?;
    let mut file = File::create("packages.json")?;
    file.write_all(json_output.as_bytes())?;
    
    println!("✅ Saved {} total packages from {} repositories to packages.json", 
             registry.total_packages, registry.repositories.len());
    
    Ok(())
}

fn load_existing_registry() -> Result<PackageRegistry, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("packages.json")?;
    let registry: PackageRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

fn load_servers_config() -> Result<ServersConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("servers.json")?;
    let config: ServersConfig = serde_json::from_str(&content)?;
    Ok(config)
}

fn parse_packages(document: &Html) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let mut packages = Vec::new();
    
    // Try different selectors for different HTML structures
    let selectors = vec![
        "pre a",  // Common for directory listings
        "a[href$='.apk']",  // APK files specifically  
        "tr td a",  // Table structure
    ];
    
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let links: Vec<_> = document.select(&selector).collect();
            
            if !links.is_empty() {
                println!("📋 Using selector: {}", selector_str);
                
                for link in links {
                    if let Some(href) = link.value().attr("href") {
                        if href.ends_with(".apk") {
                            let package = parse_package_from_link(link.inner_html().as_str(), href)?;
                            packages.push(package);
                        }
                    }
                }
                
                if !packages.is_empty() {
                    break; // Found packages with this selector, no need to try others
                }
            }
        }
    }
    
    // If HTML parsing doesn't work well, try regex fallback
    if packages.is_empty() {
        println!("🔄 HTML parsing failed, trying regex fallback...");
        packages = parse_packages_with_regex(document)?;
    }
    
    Ok(packages)
}

fn parse_package_from_link(_link_text: &str, href: &str) -> Result<Package, Box<dyn std::error::Error>> {
    let filename = href.to_string();
    
    // Extract package name and version from filename
    let (name, version) = extract_name_version(&filename)?;
    
    Ok(Package {
        name,
        version,
        size: "unknown".to_string(),  // Will try to extract if available
        date: "unknown".to_string(),   // Will try to extract if available
        filename,
    })
}

fn extract_name_version(filename: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Remove .apk extension
    let base_name = filename.replace(".apk", "");
    
    // Common Alpine package naming patterns:
    // package-version-revision.apk
    // package-version.apk
    
    // Use regex to extract name and version
    let re = regex::Regex::new(r"^(.+)-([0-9][^-]*(?:-r[0-9]+)?)$")?;
    
    if let Some(captures) = re.captures(&base_name) {
        let name = captures.get(1).unwrap().as_str().to_string();
        let version = captures.get(2).unwrap().as_str().to_string();
        Ok((name, version))
    } else {
        // Fallback: treat entire base name as package name
        Ok((base_name, "unknown".to_string()))
    }
}

fn parse_packages_with_regex(document: &Html) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let mut packages = Vec::new();
    let html_text = document.html();
    
    // Regex to find .apk files in the HTML
    let re = regex::Regex::new(r#"([a-zA-Z0-9_+-]+(?:-[0-9][^"\s]*)?\.apk)"#)?;
    
    let mut seen_packages = std::collections::HashSet::new();
    
    for cap in re.captures_iter(&html_text) {
        if let Some(filename_match) = cap.get(1) {
            let filename = filename_match.as_str();
            
            // Avoid duplicates
            if seen_packages.contains(filename) {
                continue;
            }
            seen_packages.insert(filename.to_string());
            
            let (name, version) = extract_name_version(filename)?;
            
            packages.push(Package {
                name,
                version,
                size: "unknown".to_string(),
                date: "unknown".to_string(),
                filename: filename.to_string(),
            });
        }
    }
    
    Ok(packages)
}
