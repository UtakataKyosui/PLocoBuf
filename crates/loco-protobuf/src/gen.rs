use clap::{Parser, Subcommand};
use std::path::Path;
use std::io::Result;

#[derive(Parser, Debug)]
#[command(name = "loco-protobuf")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate related files
    Generate {
        #[command(subcommand)]
        sub: GenerateCommands,
    },
}

#[derive(Subcommand, Debug)]
enum GenerateCommands {
    /// Generate a protobuf file
    Protobuf {
        /// Name of the message/file (e.g. user)
        name: String,
    },
}

pub fn handle() -> Result<bool> {
    let args: Vec<String> = std::env::args().collect();
    
    // We only care if "generate" and "protobuf" are present.
    // Clap in strict mode might fail on other loco args, so we try specific parsing or manual check first?
    // Actually, asking Clap to ignore unknown args is better.
    
    // Simple manual check for now to avoid extensive Clap config for a partial wrapper
    if args.len() >= 3 && args[1] == "generate" && args[2] == "protobuf" {
        // It matches our target command. Let's parse specific args.
        // We construct a new list to verify flags if needed, or just grab the name.
        if let Some(name) = args.get(3) {
             generate_protobuf(name)?;
             return Ok(true);
        } else {
             println!("Usage: generate protobuf <NAME>");
             return Ok(true);
        }
    }
    
    Ok(false)
}

fn generate_protobuf(name: &str) -> Result<()> {
    let filename = format!("proto/{}.proto", name);
    let path = Path::new(&filename);

    if path.exists() {
        println!("File {} already exists", filename);
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = format!(r#"syntax = "proto3";

package {};

message {}Request {{
  string id = 1;
}}

message {}Response {{
  string id = 1;
}}
"#, name, capitalize(name), capitalize(name));

    std::fs::write(path, content)?;
    println!("* {}", filename);
    println!("Don't forget to add it to your build.rs and library inclusion!");

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
