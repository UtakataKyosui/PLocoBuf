use serial_test::serial;
use std::process::Command;

#[test]
#[serial]
fn test_cli_generate_protobuf() {
    // Build the binary first (should be fast if already built)
    let status = Command::new("cargo")
        .args(&["build", "--bin", "real_app-cli"])
        .status()
        .expect("Failed to build real_app-cli");
    assert!(status.success());
    
    let bin_path = "../../target/debug/real_app-cli";

    // Clean up if exists
    let _ = std::fs::remove_file("proto/integration_test.proto");

    // Run generate command with fields
    let output = Command::new(bin_path)
        .args(&["generate", "protobuf", "integration_test", "name:string", "age:int"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("* ./proto/integration_test.proto"), "Stdout did not contain expected string. Got:\n{}", stdout);

    // Verify file exists and content
    let content = std::fs::read_to_string("proto/integration_test.proto").expect("Proto file was not created");
    assert!(content.contains("package integration_test;"));
    assert!(content.contains("message IntegrationTest {"));
    assert!(content.contains("string name = 1;"));
    assert!(content.contains("int32 age = 2;"));
    
    // Clean up
    let _ = std::fs::remove_file("proto/integration_test.proto");
}
