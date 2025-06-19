use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_find_in_empty_dir() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    
    let mut cmd = Command::cargo_bin("rust-find")?;
    cmd.arg(dir.path())
       .assert()
       .success()
       .stdout(predicate::str::is_empty());
    
    Ok(())
}

#[test]
fn test_find_with_files() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::File::create(dir.path().join("file1.txt"))?;
    std::fs::File::create(dir.path().join("file2.txt"))?;
    
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .assert()
       .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
    
    Ok(())
}

#[test]
fn test_max_depth() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let subdir = tempfile::tempdir_in(dir.path())?;
    std::fs::File::create(subdir.path().join("file.txt"))?;
    
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .arg("--max-depth")
       .arg("1")
       .assert()
       .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("file.txt")); // Should find file in subdir with max_depth=1
    
    Ok(())
}

#[test]
fn test_debug_output() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::File::create(dir.path().join("test.txt"))?;
    
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .arg("--debug")
       .assert()
       .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("test.txt"));
    
    Ok(())
}

#[test]
fn test_symlink_handling() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file = dir.path().join("file.txt");
    std::fs::File::create(&file)?;
    
    let symlink = dir.path().join("link.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&file, &symlink)?;
    
    // Test with follow links
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .arg("--follow-links")
       .assert()
       .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("file.txt"));
    #[cfg(unix)]
    assert!(stdout.contains("link.txt"));
    
    // Test without follow links
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .assert()
       .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("file.txt"));
    #[cfg(unix)]
    assert!(stdout.contains("link.txt"));
    
    Ok(())
}

#[test]
fn test_symlink_loop() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)] {
        let dir = tempdir()?;
        let link1 = dir.path().join("link1");
        let link2 = dir.path().join("link2");
        
        std::os::unix::fs::symlink(&link2, &link1)?;
        std::os::unix::fs::symlink(&link1, &link2)?;
        
        let mut cmd = Command::cargo_bin("rust-find")?;
        let output = cmd.arg(dir.path())
           .arg("--follow-links")
           .assert()
           .success();
        
        let stderr = String::from_utf8(output.get_output().stderr.clone())?;
        assert!(!stderr.is_empty()); // Just check for any error output
    }
    Ok(())
}

#[test]
fn test_broken_symlink() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)] {
        let dir = tempdir()?;
        let broken_link = dir.path().join("broken");
        std::os::unix::fs::symlink("/nonexistent/path", &broken_link)?;
        
        let mut cmd = Command::cargo_bin("rust-find")?;
        let output = cmd.arg(dir.path())
           .arg("--follow-links")
           .assert()
           .success();
        
        let stderr = String::from_utf8(output.get_output().stderr.clone())?;
        assert!(!stderr.is_empty()); // Just check for any error output
    }
    Ok(())
}

#[test]
fn test_nested_directories() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let sub1 = tempfile::tempdir_in(dir.path())?;
    let sub2 = tempfile::tempdir_in(sub1.path())?;
    
    std::fs::File::create(sub2.path().join("deep_file.txt"))?;
    std::fs::File::create(dir.path().join("top_file.txt"))?;
    
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .assert()
       .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("top_file.txt"));
    assert!(stdout.contains("deep_file.txt"));
    
    Ok(())
}

#[test]
fn test_permission_error() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let restricted_dir = dir.path().join("restricted");
    std::fs::create_dir(&restricted_dir)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&restricted_dir)?.permissions();
        perms.set_mode(0o000); // No permissions
        std::fs::set_permissions(&restricted_dir, perms)?;
    }
    
    let mut cmd = Command::cargo_bin("rust-find")?;
    let output = cmd.arg(dir.path())
       .assert()
       .success();
    
    #[cfg(unix)]
    {
        let stderr = String::from_utf8(output.get_output().stderr.clone())?;
        assert!(!stderr.is_empty()); // Just check for any error output
    }
    
    Ok(())
}