use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

static STORE: Mutex<()> = Mutex::new(());
static NEXT: AtomicU32 = AtomicU32::new(0);
const MAGIC: &str = "catplay-phone-switch-v1";

pub(super) fn path(cache: &str) -> PathBuf {
    PathBuf::from(format!("{cache}.switch-intent"))
}

fn parse(data: &str) -> io::Result<(String, u8)> {
    let fields: Vec<_> = data.lines().collect();
    if fields.len() != 3
        || fields[0] != MAGIC
        || fields[1].split(':').count() != 6
        || !fields[1]
            .split(':')
            .all(|part| part.len() == 2 && part.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid switch intent"));
    }
    let boots: u8 = fields[2]
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid boot budget"))?;
    if boots > 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid boot budget"));
    }
    Ok((fields[1].to_owned(), boots))
}

fn read(path: &Path) -> io::Result<Option<(String, u8)>> {
    match fs::read_to_string(path) {
        Ok(data) => parse(&data).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn write(path: &Path, target: &str, boots: u8) -> io::Result<()> {
    let data = format!("{MAGIC}\n{target}\n{boots}\n");
    parse(&data)?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("switch intent has no parent"))?;
    let mut temporary = None;
    for _ in 0..16 {
        let name = parent.join(format!(
            ".catplay-switch-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&name) {
            Ok(file) => {
                temporary = Some((name, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (name, mut file) = temporary.ok_or_else(|| io::Error::other("cannot allocate switch intent temporary file"))?;
    let result = (|| {
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        fs::rename(&name, path)?;
        File::open(parent)?.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(name);
    }
    result
}

pub(super) fn save(path: &Path, target: &str) -> io::Result<()> {
    let _guard = STORE.lock().unwrap();
    write(path, target, 2)
}

pub(super) fn resume(path: &Path) -> io::Result<Option<String>> {
    let _guard = STORE.lock().unwrap();
    let Some((target, boots)) = read(path)? else {
        return Ok(None);
    };
    if boots == 0 {
        fs::remove_file(path)?;
        File::open(path.parent().unwrap())?.sync_all()?;
        return Ok(None);
    }
    write(path, &target, boots - 1)?;
    Ok(Some(target))
}

pub(super) fn clear(path: &Path, expected: &str) -> io::Result<()> {
    let _guard = STORE.lock().unwrap();
    if let Some((target, _)) = read(path)? {
        if !target.eq_ignore_ascii_case(expected) {
            return Err(io::Error::other("different switch intent is active"));
        }
        fs::remove_file(path)?;
        File::open(path.parent().unwrap())?.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "catplay-intent-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        dir
    }
    #[test]
    fn desired_phone_survives_two_boots_then_falls_back_without_a_loop() {
        let dir = directory();
        let path = dir.join("intent");
        save(&path, "AA:00:00:00:00:02").unwrap();
        assert_eq!(resume(&path).unwrap().as_deref(), Some("AA:00:00:00:00:02"));
        assert_eq!(resume(&path).unwrap().as_deref(), Some("AA:00:00:00:00:02"));
        assert_eq!(resume(&path).unwrap(), None);
        assert!(!path.exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn completion_cannot_clear_a_newer_target_and_success_removes_the_intent() {
        let dir = directory();
        let path = dir.join("intent");
        save(&path, "AA:00:00:00:00:02").unwrap();
        assert!(clear(&path, "AA:00:00:00:00:01").is_err());
        assert!(path.exists());
        clear(&path, "AA:00:00:00:00:02").unwrap();
        assert!(!path.exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn failed_storage_or_malformed_intent_is_reported_before_handover() {
        let dir = directory();
        let path = dir.join("intent");
        assert!(save(&dir.join("missing/intent"), "AA:00:00:00:00:02").is_err());
        fs::write(&path, "broken").unwrap();
        assert!(resume(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn switch_intent_is_private_and_cannot_overwrite_the_last_phone_cache() {
        let dir = directory();
        let cache = dir.join("last-phone");
        fs::write(&cache, "AA:00:00:00:00:01\n").unwrap();
        let intent = path(cache.to_str().unwrap());
        save(&intent, "AA:00:00:00:00:02").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&intent).unwrap().permissions().mode() & 0o777, 0o600);
        }
        assert_eq!(fs::read_to_string(&cache).unwrap(), "AA:00:00:00:00:01\n");
        clear(&intent, "AA:00:00:00:00:02").unwrap();
        assert!(cache.exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
