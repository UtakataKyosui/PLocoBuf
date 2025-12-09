use std::path::Path;
use std::io::Result;

pub fn handle() -> Result<bool> {
    handle_args(std::env::args())
}

pub fn handle_args<I, T>(args: I) -> Result<bool>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    
    if args.len() >= 3 && args[1] == "generate" && args[2] == "protobuf" {
        if let Some(name) = args.get(3) {
             let fields = if args.len() > 4 {
                 &args[4..]
             } else {
                 &[]
             };
             generate_protobuf(name, fields)?;
             return Ok(true);
        } else {
             println!("Usage: generate protobuf <NAME> [FIELDS...]");
             return Ok(true);
        }
    }
    
    Ok(false)
}

fn generate_protobuf(name: &str, fields: &[String]) -> Result<()> {
    // For integration testing we allow writing to proto/ relative to current dir
    generate_protobuf_at(name, ".", fields)
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
        rules.push_str(&format!("  {} {} = {};\n", proto_type, field_name, i + 1));
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
        assert!(content.contains("string name = 1;"));
        assert!(content.contains("int32 age = 2;"));
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
}
