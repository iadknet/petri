#[test]
fn retired_campaign_refuses_before_opening_output_paths() {
    let directory =
        std::env::temp_dir().join(format!("petri-retired-discovery-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let summary = directory.join("summary.json");
    // A directory is deliberately not a writable raw file: even the old command
    // fails before constructing native panels or generating any proposals.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_v3-cli"))
        .args([
            "input-discovery",
            "--feature",
            "t20-f09-inherited-discovery-qualification",
            "--out",
        ])
        .arg(&directory)
        .arg("--summary-out")
        .arg(&summary)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("retired"), "{error}");
    assert!(
        error.contains("cb254ca9f89d158a1da6a9a3e62b3ac4994dba67"),
        "{error}"
    );
    assert!(!summary.exists());
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    std::fs::remove_dir(directory).unwrap();
}
