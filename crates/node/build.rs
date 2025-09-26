use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=static/styles.css");
    println!("cargo:rerun-if-changed=src/templates.rs");
    
    // Create output directory
    let dist_dir = Path::new("static/dist");
    fs::create_dir_all(dist_dir).expect("Failed to create dist directory");
    
    // Try to build CSS with Tailwind
    let tailwind_result = Command::new("tailwindcss")
        .args(&[
            "-i", "static/styles.css",
            "-o", "static/dist/app.css",
            "--content", "src/**/*.rs",
            "--minify"
        ])
        .output();
    
    match tailwind_result {
        Ok(output) if output.status.success() => {
            println!("Successfully built CSS with Tailwind");
        }
        Ok(output) => {
            eprintln!("Tailwind CSS build failed: {}", String::from_utf8_lossy(&output.stderr));
            eprintln!("Using fallback CSS");
        }
        Err(e) => {
            eprintln!("Warning: Failed to run tailwindcss: {}. Using fallback CSS.", e);
        }
    }
}