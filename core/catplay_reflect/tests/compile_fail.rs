use std::{fs, path::PathBuf};

#[test]
fn compile_fail() {
    let mut config = compiletest_rs::Config::default();
    config.mode = "compile-fail".parse().expect("valid compiletest mode");
    config.src_base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ui");
    assert!(
        !compiletest_rs::make_tests(&config).is_empty(),
        "compile-fail fixture directory is empty"
    );
    config.edition = Some("2021".to_owned());
    config.link_deps();
    let deps = std::env::current_exe()
        .expect("test executable path")
        .parent()
        .expect("test executable directory")
        .to_owned();
    let libraries: Vec<_> = fs::read_dir(&deps)
        .expect("Cargo dependency directory")
        .map(|entry| entry.expect("dependency entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("libcatplay_reflect-") && name.ends_with(".rlib"))
        })
        .collect();
    let [library] = libraries.as_slice() else {
        panic!("expected one catplay_reflect rlib in {}: {libraries:?}", deps.display());
    };
    config.target_rustcflags = Some(format!(
        "{} --extern catplay_reflect={} --crate-type=lib --emit=metadata",
        config.target_rustcflags.take().unwrap_or_default(),
        library.display(),
    ));
    compiletest_rs::run_tests(&config);
}
