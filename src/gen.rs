use std::path::{Path, PathBuf};
use std::io::Result;
use std::env;

/// Helper to extract name and fields from args
fn extract_name_and_fields<'a>(args: &'a [String], start_idx: usize) -> Option<(&'a str, &'a [String])> {
    args.get(start_idx).map(|name| {
        let fields = if args.len() > start_idx + 1 {
            &args[start_idx + 1..]
        } else {
            &[]
        };
        (name.as_str(), fields)
    })
}

/// Find the project root by looking for Cargo.toml
fn find_project_root() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;
    
    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current_dir);
        }
        
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find Cargo.toml in current directory or any parent directory"
                ));
            }
        }
    }
}
pub fn handle() -> Result<bool> {
    handle_args(std::env::args())
}

pub fn handle_args<I, T>(args: I) -> Result<bool>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    
    if args.len() >= 3 && args[1] == "generate" {
        match args[2].as_str() {
            "protobuf" => {
                if let Some((name, fields)) = extract_name_and_fields(&args, 3) {
                     generate_protobuf(name, fields)?;
                     return Ok(true);
                } else {
                     eprintln!("Usage: generate protobuf <NAME> [FIELDS...]");
                     return Ok(true);
                }
            }
            "migration_from_proto" => {
                 if let Some(proto_name) = args.get(3) {
                     generate_migration_from_proto(proto_name)?;
                     return Ok(true);
                 } else {
                     eprintln!("Usage: generate migration_from_proto <PROTO_NAME>");
                     return Ok(true);
                 }
            }
            "proto_model" => {
                if let Some((name, fields)) = extract_name_and_fields(&args, 3) {
                    generate_proto_model(name, fields)?;
                    return Ok(true);
                } else {
                    eprintln!("Usage: generate proto_model <NAME> [FIELDS...]");
                    return Ok(true);
                }
            }
            "proto_controller" => {
                if let Some(name) = args.get(3) {
                    generate_proto_controller(name)?;
                    return Ok(true);
                } else {
                    eprintln!("Usage: generate proto_controller <NAME>");
                    return Ok(true);
                }
            }
            "proto_scaffold" => {
                if let Some((name, fields)) = extract_name_and_fields(&args, 3) {
                    generate_proto_scaffold(name, fields)?;
                    return Ok(true);
                } else {
                    eprintln!("Usage: generate proto_scaffold <NAME> [FIELDS...]");
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    
    Ok(false)
}

fn generate_protobuf(name: &str, fields: &[String]) -> Result<()> {
    let project_root = find_project_root()?;
    let root = project_root.to_str().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Project root path contains invalid UTF-8"
    ))?;
    generate_protobuf_at(name, root, fields)
}

fn generate_migration_from_proto(proto_name: &str) -> Result<()> {
    let project_root = find_project_root()?;
    let proto_path = project_root.join("proto").join(format!("{}.proto", proto_name));
    
    if !proto_path.exists() {
        eprintln!("Proto file not found: {}", proto_path.display());
        return Ok(());
    }

    let content = std::fs::read_to_string(&proto_path)?;
    let (message_name, fields) = parse_proto(&content).ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse proto"))?;
    
    // Convert proto fields to Loco migration format, preserving required/optional
    let args: Vec<String> = fields.into_iter().map(|(name, proto_type, is_optional)| {
        let loco_type = map_proto_type_to_loco(&proto_type);
        // Add ! modifier for required fields (not optional)
        if is_optional {
            format!("{}:{}", name, loco_type)
        } else {
            format!("{}:{}!", name, loco_type)
        }
    }).collect();
    
    println!("Generating migration for {} with fields: {:?}", message_name, args);
    
    let status = std::process::Command::new("cargo")
        .args(&["loco", "generate", "migration", &format!("Create{}", pluralize(&capitalize(&message_name)))])
        .args(&args)
        .status()?;

    if !status.success() {
        eprintln!("Failed to run cargo loco generate migration");
    }

    Ok(())
}

fn generate_proto_model(name: &str, fields: &[String]) -> Result<()> {
    println!("Generating proto_model for: {}", name);
    
    // Step 1: Generate .proto file
    generate_protobuf(name, fields)?;
    
    // Step 2: Generate migration from the proto file
    generate_migration_from_proto(name)?;
    
    // Step 3: Auto-run migration
    println!("Running migrations...");
    let status = std::process::Command::new("cargo")
        .args(&["loco", "db", "migrate"])
        .status()?;

    if !status.success() {
        eprintln!("Warning: Migration failed. You may need to run 'cargo loco db migrate' manually.");
    } else {
        println!("✓ Migrations applied successfully");
    }
    
    println!("\n✓ Proto model '{}' generated successfully!", name);
    println!("  - proto/{}.proto", name);
    println!("  - Migration file created");
    println!("  - Database migrated");
    
    Ok(())
}

fn generate_proto_controller(name: &str) -> Result<()> {
    println!("Generating proto_controller for: {}", name);
    
    let project_root = find_project_root()?;
    let snake_name = name.to_lowercase();
    let pascal_name = capitalize(name);
    
    let controller_content = format!(r#"#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::response::Result;
use loco_protobuf::Protobuf;
use loco_rs::prelude::*;
use crate::proto::{}::{{{}}}; 

pub async fn create(
    State(ctx): State<AppContext>,
    Protobuf(req): Protobuf<{}>,
) -> Result<Protobuf<{}>> {{
    // TODO: Implement create logic
    Ok(Protobuf({}::default()))
}}

pub async fn get(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Protobuf<{}>> {{
    // TODO: Implement get logic
    Ok(Protobuf({}::default()))
}}

pub fn routes() -> Routes {{
    Routes::new()
        .prefix("api/{}")
        .add("/", post(create))
        .add("/:id", get(get))
}}
"#, snake_name, pascal_name, pascal_name, pascal_name, pascal_name, pascal_name, pascal_name, pluralize(&snake_name));
    
    // Write controller file
    let controller_dir = project_root.join("src").join("controllers");
    std::fs::create_dir_all(&controller_dir)?;
    let controller_path = controller_dir.join(format!("{}.rs", snake_name));
    std::fs::write(&controller_path, controller_content)?;
    
    println!("\n✓ Proto controller '{}' generated successfully!", name);
    println!("  - src/controllers/{}.rs", snake_name);
    println!("\nNext steps:");
    println!("  1. Add 'pub mod {};' to src/controllers/mod.rs", snake_name);
    println!("  2. Add route in src/app.rs: .add_route(controllers::{}::routes())", snake_name);
    println!("  3. Define proto messages in proto/{}.proto", snake_name);
    println!("  4. Implement controller logic");
    
    Ok(())
}

fn generate_proto_scaffold(name: &str, fields: &[String]) -> Result<()> {
    println!("Generating proto_scaffold for: {}", name);
    println!("This will create:");
    println!("  - .proto file with message definitions");
    println!("  - Database migration");
    println!("  - Controller with CRUD endpoints\n");
    
    // Step 1: Generate proto model (includes .proto + migration + auto-migrate)
    generate_proto_model(name, fields)?;
    
    // Step 2: Generate proto controller
    generate_proto_controller(name)?;
    
    println!("\n✓ Proto scaffold '{}' completed successfully!", name);
    println!("\nGenerated files:");
    println!("  - proto/{}.proto", name);
    println!("  - Migration file");
    println!("  - src/controllers/{}.rs", name);
    println!("\nNext steps:");
    println!("  1. Review generated controller in src/controllers/{}.rs", name);
    println!("  2. Implement business logic in controller methods");
    println!("  3. Test your endpoints!");
    
    Ok(())
}

fn map_proto_type_to_loco(proto_type: &str) -> &str {
    match proto_type {
        "int32" => "int",
        "int64" => "bigint",
        "uint32" => "unsigned",
        "uint64" => "big_unsigned",
        "float" => "float",
        "double" => "double",
        "bool" => "bool",
        "string" => "string",
        "bytes" => "blob",
         _ => "string",
    }
}

fn parse_proto(content: &str) -> Option<(String, Vec<(String, String, bool)>)> {
    let mut message_name = String::new();
    let mut fields = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("message ") {
            message_name = line.trim_start_matches("message ").trim_end_matches(" {").to_string();
        } else if !message_name.is_empty() && line.contains(" = ") {
            // Parse: [optional] type name = tag;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let (type_name, field_name, is_optional) = if parts[0] == "optional" && parts.len() >= 4 {
                    // optional type name = tag;
                    (parts[1], parts[2], true)
                } else {
                    // type name = tag; (required in proto3)
                    (parts[0], parts[1], false)
                };
                fields.push((field_name.to_string(), type_name.to_string(), is_optional));
            }
        }
    }
    
    if message_name.is_empty() {
        None
    } else {
        Some((message_name, fields))
    }
}

fn generate_protobuf_at(name: &str, root: &str, fields: &[String]) -> Result<()> {
    let filename = format!("{}/proto/{}.proto", root, name);
    let path = Path::new(&filename);

    if path.exists() {
        println!("File {} already exists", filename);
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut rules = String::new();
    for (i, field) in fields.iter().enumerate() {
        let parts: Vec<&str> = field.split(':').collect();
        let field_name = parts[0];
        let raw_type = parts.get(1).unwrap_or(&"string");
        
        // Check if field is required (ends with !) or unique (ends with ^)
        let is_required = raw_type.ends_with('!');
        let is_unique = raw_type.ends_with('^');
        let clean_type = raw_type.trim_end_matches(|c| c == '!' || c == '^');
        
        let proto_type = match clean_type {
            "int" | "integer" | "tiny_integer" | "small_integer" | "int32" | "int4" | "small_int" => "int32",
            "bigint" | "big_integer" | "int64" | "int8" | "big_int" => "int64",
            "unsigned" | "small_unsigned" => "uint32",
            "big_unsigned" => "uint64",
            "float" => "float",
            "double" | "decimal" | "decimal_len" | "money" => "double",
            "bool" | "boolean" => "bool",
            "uuid" | "string" | "text" | "json" | "jsonb" => "string",
            "blob" | "binary_len" | "var_binary" => "bytes",
            "date" | "date_time" | "tstz" | "timestamp" => "int64", 
             _ => "string",
        };
        
        // Add 'optional' keyword for nullable fields (proto3 syntax)
        // Note: unique (^) fields are typically required in DB but we preserve the modifier
        let field_modifier = if is_required || is_unique { "" } else { "optional " };
        rules.push_str(&format!("  {}{} {} = {};\n", field_modifier, proto_type, field_name, i + 1));
    }
    
    // Default field if none provided
    if rules.is_empty() {
        rules.push_str("  string id = 1;\n");
    }

    let content = format!(r#"syntax = "proto3";

package {};

message {} {{
{}
}}
"#, name, capitalize(name), rules.trim_end());

    std::fs::write(path, content)?;
    println!("* {}", filename);
    println!("Don't forget to add it to your build.rs and library inclusion!");

    Ok(())
}

fn capitalize(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

fn pluralize(s: &str) -> String {
    if s.ends_with('y') && !s.ends_with("ay") && !s.ends_with("ey") && !s.ends_with("iy") && !s.ends_with("oy") && !s.ends_with("uy") {
        format!("{}ies", &s[..s.len()-1])
    } else if s.ends_with("ss") || s.ends_with("us") || s.ends_with("sh") || s.ends_with("ch") || s.ends_with('x') || s.ends_with('z') {
        // status -> statuses, class -> classes, box -> boxes
        format!("{}es", s)
    } else if s.ends_with('s') {
        // Words already ending in 's' (but not 'ss' or 'us') - just return as is
        // e.g., products, users (already plural)
        s.to_string()
    } else {
        format!("{}s", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_generate_protobuf_content() {
        let temp_dir = tempfile::Builder::new().prefix("loco_proto_test").tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap();
        
        let fields = vec!["name:string".to_string(), "age:int".to_string()];
        generate_protobuf_at("unit_test", root, &fields).expect("Failed to generate protobuf");
        
        let file_path = temp_dir.path().join("proto/unit_test.proto");
        assert!(file_path.exists());
        
        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("package unit_test;"));
        assert!(content.contains("message UnitTest {"));
        assert!(content.contains("optional string name = 1;"));
        assert!(content.contains("optional int32 age = 2;"));
    }
    
    #[test]
    fn test_generate_protobuf_default() {
        let temp_dir = tempfile::Builder::new().prefix("loco_proto_test_default").tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap();
        
        generate_protobuf_at("default_test", root, &[]).expect("Failed to generate protobuf");
        
        let file_path = temp_dir.path().join("proto/default_test.proto");
        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("string id = 1;"));
    }

    #[test]
    fn test_handle_args_logic() {
        // Not a generate command
        assert_eq!(handle_args(vec!["bin", "server", "start"]).unwrap(), false);
        
        // Correct command but missing arg handled by print usage (returns true)
        assert_eq!(handle_args(vec!["bin", "generate", "protobuf"]).unwrap(), true);
    }

    #[test]
    fn test_generate_protobuf_types() {
        let temp_dir = tempfile::Builder::new().prefix("loco_proto_types").tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap();
        
        let fields = vec![
            "param_i:integer".to_string(),
            "param_bi:bigint".to_string(),
            "param_s:string".to_string(),
            "param_t:text".to_string(),
            "param_u:uuid".to_string(),
            "param_ts:timestamp".to_string(),
            "param_j:jsonb".to_string(),
        ];
        generate_protobuf_at("types_test", root, &fields).expect("Failed to generate protobuf");
        
        let file_path = temp_dir.path().join("proto/types_test.proto");
        let content = fs::read_to_string(file_path).unwrap();
        
        assert!(content.contains("int32 param_i = 1;"));
        assert!(content.contains("int64 param_bi = 2;"));
        assert!(content.contains("string param_s = 3;"));
        assert!(content.contains("string param_t = 4;"));
        assert!(content.contains("string param_u = 5;"));
        assert!(content.contains("int64 param_ts = 6;"));
        assert!(content.contains("string param_j = 7;"));
    }

    #[test]
    fn test_generate_protobuf_loco_types() {
        let temp_dir = tempfile::Builder::new().prefix("loco_proto_loco_types").tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap();
        
        let fields = vec![
            "req_str:string!".to_string(),
            "uniq_uuid:uuid^".to_string(),
            "opt_int:int".to_string(),
            "tstz_field:tstz".to_string(),
            "unsigned_val:unsigned".to_string(),
            "blob_data:blob".to_string(),
            "money_val:money".to_string(),
            "array_val:array".to_string(), // fallback to string
        ];
        generate_protobuf_at("loco_types", root, &fields).expect("Failed to generate protobuf");
        
        let file_path = temp_dir.path().join("proto/loco_types.proto");
        let content = fs::read_to_string(file_path).unwrap();
        
        assert!(content.contains("string req_str = 1;"));
        assert!(content.contains("string uniq_uuid = 2;"));
        assert!(content.contains("int32 opt_int = 3;"));
        assert!(content.contains("int64 tstz_field = 4;"));
        assert!(content.contains("uint32 unsigned_val = 5;"));
        assert!(content.contains("bytes blob_data = 6;"));
        assert!(content.contains("double money_val = 7;"));
        assert!(content.contains("string array_val = 8;"));
    }
    #[test]
    fn test_parse_proto() {
        let content = r#"syntax = "proto3";

package user;

message User {
  string name = 1;
  optional int32 age = 2;
}
"#;
        let result = parse_proto(content);
        assert!(result.is_some());
        let (name, fields) = result.unwrap();
        assert_eq!(name, "User");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], ("name".to_string(), "string".to_string(), false)); // required
        assert_eq!(fields[1], ("age".to_string(), "int32".to_string(), true)); // optional
    }

    #[test]
    fn test_map_type() {
        assert_eq!(map_proto_type_to_loco("int32"), "int");
        assert_eq!(map_proto_type_to_loco("int64"), "bigint");
        assert_eq!(map_proto_type_to_loco("string"), "string");
        assert_eq!(map_proto_type_to_loco("bytes"), "blob");
    }
}
