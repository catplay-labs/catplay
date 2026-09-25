use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct UdcHelper {}

impl UdcHelper {
    pub fn requires_udc(phone: bool, pinned: bool) -> bool {
        (phone && !pinned) || (!phone && pinned)
    }

    pub fn resolve_udc_name(udc: Option<&str>) -> io::Result<Option<String>> {
        let ret = match udc {
            None => Self::default_udc_name().ok(),
            Some(udc) => Self::udc_names()?.into_iter().find(|name| name == udc),
        };
        Ok(ret)
    }

    fn udc_names() -> io::Result<Vec<String>> {
        let class_dir = Path::new("/sys/class");
        if !class_dir.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "sysfs is not available"));
        }

        let udc_dir = class_dir.join("udc");
        if !udc_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut udcs = Vec::new();
        for entry in fs::read_dir(&udc_dir)? {
            let Ok(entry) = entry else { continue };
            if let Some(name) = Self::path_file_name(entry.path()) {
                udcs.push(name);
            }
        }

        Ok(udcs)
    }

    fn default_udc_name() -> io::Result<String> {
        let mut udcs = Self::udc_names()?;
        udcs.sort();
        udcs.into_iter()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no USB device controller (UDC) available"))
    }

    fn path_file_name(path: PathBuf) -> Option<String> {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }
}
