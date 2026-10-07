use super::{archive::KeyedArchive, generate_lingoes, ir::Protocol, preview::Expander};
use anyhow::{Context, Result, bail};
use catplay_lingo as _;
use plist::Value;
use std::{env, fs, io::Cursor, path::PathBuf, process::Command};

pub(crate) fn run_cli() -> Result<()> {
    let input = env::args()
        .nth(1)
        .context("usage: catplay_lingo <Default.lktspec> [output directory]")?;
    let output = env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lingos"));
    let bytes = fs::read(&input).with_context(|| format!("failed to read {input}"))?;
    let root = Value::from_reader(Cursor::new(&bytes)).context("failed to parse bplist")?;
    let archive = KeyedArchive::new(&root)?;
    let mut protocol = Protocol::parse(&archive)?;
    super::patches::apply(&mut protocol)?;
    fs::create_dir_all(&output).with_context(|| format!("failed to create {}", output.display()))?;
    let preview_dir = output.join("preview");
    fs::create_dir_all(&preview_dir).with_context(|| format!("failed to create {}", preview_dir.display()))?;
    let expander = Expander::new()?;
    let files = generate_lingoes(&protocol)?;
    for (name, source) in &files {
        let path = output.join(name);
        fs::write(&path, source).with_context(|| format!("failed to write {}", path.display()))?;
        let status = Command::new(env::var_os("RUSTFMT").unwrap_or_else(|| "rustfmt".into()))
            .arg("--edition")
            .arg("2024")
            .arg(&path)
            .status()
            .with_context(|| format!("failed to format {}", path.display()))?;
        if !status.success() {
            bail!("rustfmt failed for {}", path.display());
        }
    }
    for (name, _) in &files {
        if name == "strings.rs" {
            continue;
        }
        let expanded = expander.expand(name, &output.join(name))?;
        let preview_path = preview_dir.join(name.replace(".rs", ".txt"));
        fs::write(&preview_path, expanded).with_context(|| format!("failed to write {}", preview_path.display()))?;
    }
    Ok(())
}
