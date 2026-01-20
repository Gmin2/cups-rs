use std::process::Command;
use std::str;

use cups_rs::{get_all_destinations, get_default_destination};

fn cups_available() -> bool {
    get_all_destinations().is_ok()
}

fn get_test_printer() -> Option<String> {
    if let Ok(dest) = get_default_destination() {
        return Some(dest.name);
    }
    if let Ok(dests) = get_all_destinations() {
        return dests.into_iter().next().map(|d| d.name);
    }
    None
}

#[test]
fn test_lp_submit_and_lpstat_verify() {
    if !cups_available() {
        println!("CUPS not available, skipping test");
        return;
    }

    let printer = match get_test_printer() {
        Some(p) => p,
        None => {
            println!("No printers found, skipping test");
            return;
        }
    };

    println!("Using printer: {}", printer);

    // tmp file.txt
    let file_content = "Hello, Integration Test!";
    let temp_file = tempfile::Builder::new()
        .prefix("test_lp_")
        .suffix(".txt")
        .tempfile()
        .expect("Failed to create temp file");
    std::fs::write(temp_file.path(), file_content).expect("Failed to write temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Submit job using `lp` example
    let lp_output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--example",
            "lp",
            "--",
            "-d",
            &printer,
            file_path,
        ])
        .output()
        .expect("Failed to run lp example");

    assert!(
        lp_output.status.success(),
        "lp example failed: {}",
        str::from_utf8(&lp_output.stderr).unwrap()
    );

    let lp_stdout = str::from_utf8(&lp_output.stdout).unwrap();
    println!("lp output: {}", lp_stdout);

    assert!(
        lp_stdout.contains("request id is"),
        "lp output missing request id"
    );
    assert!(
        lp_stdout.contains(&printer),
        "lp output missing printer name"
    );

    // Extract Job ID
    let job_id_part = lp_stdout
        .split("is ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .expect("Failed to parse job ID");

    println!("Submitted Job ID: {}", job_id_part);

    // Verify with `lpstat` example
    let mut found = false;
    let mut lpstat_stdout = String::new();

    for _ in 0..15 {
        let lpstat_output = Command::new("cargo")
            .args(&[
                "run",
                "--quiet",
                "--example",
                "lpstat",
                "--",
                "-W",
                "all",
                "-o",
                &printer,
            ])
            .output()
            .expect("Failed to run lpstat example");

        assert!(lpstat_output.status.success(), "lpstat example failed");
        
        lpstat_stdout = str::from_utf8(&lpstat_output.stdout).unwrap().to_string();
        
        if lpstat_stdout.contains(job_id_part) {
            found = true;
            break;
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    if !found {
        println!("lpstat output:\n{}", lpstat_stdout);
        panic!("Job ID {} not found in active or completed queue after 15 seconds", job_id_part);
    }
}
