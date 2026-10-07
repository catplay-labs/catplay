//! Produce macro expansion previews for generated lingos.

use anyhow::{Context, Result, bail};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

pub(crate) struct Expander {
    deps: PathBuf,
    lingo_rlib: PathBuf,
    bitflags_rlib: PathBuf,
    field_project_rlib: PathBuf,
    temporary: PathBuf,
}

impl Expander {
    pub(crate) fn new() -> Result<Self> {
        let executable = env::current_exe().context("cannot locate generator executable")?;
        let parent = executable
            .parent()
            .context("generator executable has no parent directory")?;
        let deps = if parent.file_name().is_some_and(|name| name == "deps") {
            parent.to_path_buf()
        } else {
            parent.join("deps")
        };
        let lingo_rlib = latest_rlib(&deps, "libcatplay_lingo-")?;
        let bitflags_rlib = latest_rlib(&deps, "libbitflags-")?;
        let field_project_rlib = latest_rlib(&deps, "libcatplay_reflect-")?;
        let temporary = env::temp_dir().join(format!(
            "catplay-lingo-preview-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos()
        ));
        fs::create_dir(&temporary).with_context(|| format!("cannot create {}", temporary.display()))?;
        Ok(Self {
            deps,
            lingo_rlib,
            bitflags_rlib,
            field_project_rlib,
            temporary,
        })
    }

    pub(crate) fn expand(&self, name: &str, source_path: &Path) -> Result<String> {
        let source = fs::read_to_string(source_path).with_context(|| format!("cannot read {}", source_path.display()))?;
        let clean_path = self.temporary.join(name);
        fs::write(&clean_path, source)?;
        let harness_path = self.temporary.join(format!("harness_{name}"));
        let module = name
            .strip_suffix(".rs")
            .context("lingo filename must end in .rs")?;
        let strings_path = source_path
            .parent()
            .context("lingo source has no parent directory")?
            .join("strings.rs")
            .canonicalize()
            .context("cannot find generated strings.rs")?;
        let harness = format!(
            "extern crate catplay_lingo;\nextern crate bitflags;\nextern crate catplay_reflect;\npub use catplay_lingo::*;\n#[path = {:?}] mod strings;\nmod {module} {{\n    include!({:?});\n}}\n",
            strings_path.display().to_string(),
            clean_path.display().to_string()
        );
        fs::write(&harness_path, harness)?;
        let output = Command::new(env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .arg("--edition=2024")
            .arg("--crate-type=lib")
            .arg("-Zunpretty=expanded")
            .arg("-L")
            .arg(format!("dependency={}", self.deps.display()))
            .arg("--extern")
            .arg(format!("catplay_lingo={}", self.lingo_rlib.display()))
            .arg("--extern")
            .arg(format!("bitflags={}", self.bitflags_rlib.display()))
            .arg("--extern")
            .arg(format!("catplay_reflect={}", self.field_project_rlib.display()))
            .arg(&harness_path)
            .env("RUSTC_BOOTSTRAP", "1")
            .output()
            .with_context(|| format!("failed to run rustc for {name}"))?;
        if !output.status.success() {
            bail!("macro expansion failed for {name}: {}", String::from_utf8_lossy(&output.stderr));
        }
        let expanded = String::from_utf8(output.stdout).context("rustc produced non-UTF-8 expansion")?;
        Ok(format!("// @generated macro expansion preview for {name}\n\n{expanded}"))
    }
}

impl Drop for Expander {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temporary);
    }
}

fn latest_rlib(deps: &Path, prefix: &str) -> Result<PathBuf> {
    let mut latest: Option<(PathBuf, SystemTime)> = None;
    for entry in fs::read_dir(deps).with_context(|| format!("cannot read {}", deps.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(prefix) || !name.ends_with(".rlib") {
            continue;
        }
        let modified = entry.metadata()?.modified()?;
        if latest.as_ref().is_none_or(|(_, time)| modified > *time) {
            latest = Some((path, modified));
        }
    }
    latest
        .map(|(path, _)| path)
        .with_context(|| format!("cannot find {prefix}*.rlib in {}", deps.display()))
}
