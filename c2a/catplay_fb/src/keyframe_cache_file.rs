use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const KEYFRAME_CACHE_MAGIC: &[u8; 8] = b"CPKFC\x00\x01\x00";

pub struct KeyframeCacheFile {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyframeCacheKey {
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyframeCacheEntry {
    pub sps_pps: Vec<u8>,
    pub keyframe: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyframeCacheHeader {
    key: KeyframeCacheKey,
    sps_pps_len: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum KeyframeCacheFileError {
    #[error("Keyframe cache file not found")]
    NotFound,
    #[error("Keyframe cache file is out of date")]
    OutOfDate,
    #[error("Invalid keyframe cache file: {0}")]
    InvalidFormat(&'static str),
    #[error("Invalid keyframe cache hash: {0}")]
    InvalidHash(#[from] std::string::FromUtf8Error),
    #[error("Failed to access keyframe cache file: {0}")]
    Io(#[from] std::io::Error),
}

impl KeyframeCacheFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self, expected: &KeyframeCacheKey) -> Result<KeyframeCacheEntry, KeyframeCacheFileError> {
        let buf = match std::fs::read(&self.path) {
            Ok(buf) => buf,
            Err(err) if err.kind() == ErrorKind::NotFound => return Err(KeyframeCacheFileError::NotFound),
            Err(err) => return Err(err.into()),
        };

        let mut cursor = KeyframeCacheCursor::new(&buf);
        let magic = cursor.take(KEYFRAME_CACHE_MAGIC.len())?;
        if magic != KEYFRAME_CACHE_MAGIC {
            return Err(KeyframeCacheFileError::InvalidFormat("bad magic"));
        }

        let header_len = cursor.read_u32()? as usize;
        let expected_header_crc = cursor.read_u32()?;
        let expected_content_crc = cursor.read_u32()?;
        let header_end = cursor
            .position()
            .checked_add(header_len)
            .ok_or(KeyframeCacheFileError::InvalidFormat("header length overflow"))?;
        if header_end > buf.len() {
            return Err(KeyframeCacheFileError::InvalidFormat("truncated header"));
        }

        let header_bytes = &buf[cursor.position()..header_end];
        if crc32fast::hash(header_bytes) != expected_header_crc {
            return Err(KeyframeCacheFileError::InvalidFormat("header CRC mismatch"));
        }

        let body = &buf[header_end..];
        if crc32fast::hash(body) != expected_content_crc {
            return Err(KeyframeCacheFileError::InvalidFormat("content CRC mismatch"));
        }

        let mut header_cursor = KeyframeCacheCursor::new(header_bytes);
        let header = KeyframeCacheHeader::decode(&mut header_cursor)?;
        if header_cursor.remaining() != 0 {
            return Err(KeyframeCacheFileError::InvalidFormat("trailing header bytes"));
        }
        if &header.key != expected {
            return Err(KeyframeCacheFileError::OutOfDate);
        }

        let sps_pps_len = header.sps_pps_len as usize;
        if sps_pps_len > body.len() {
            return Err(KeyframeCacheFileError::InvalidFormat("truncated SPS/PPS buffer"));
        }

        Ok(KeyframeCacheEntry {
            sps_pps: body[..sps_pps_len].to_vec(),
            keyframe: body[sps_pps_len..].to_vec(),
        })
    }

    pub fn store(&self, key: &KeyframeCacheKey, entry: &KeyframeCacheEntry) -> Result<(), KeyframeCacheFileError> {
        if entry.sps_pps.len() > u32::MAX as usize {
            return Err(KeyframeCacheFileError::InvalidFormat("SPS/PPS buffer too large"));
        }

        let header = KeyframeCacheHeader {
            key: key.clone(),
            sps_pps_len: entry.sps_pps.len() as u32,
        };
        let header = header.encode()?;
        let header_len = u32::try_from(header.len()).map_err(|_| KeyframeCacheFileError::InvalidFormat("header too large"))?;
        let mut content_hasher = crc32fast::Hasher::new();
        content_hasher.update(&entry.sps_pps);
        content_hasher.update(&entry.keyframe);
        let content_crc = content_hasher.finalize();

        let mut buf = Vec::with_capacity(KEYFRAME_CACHE_MAGIC.len() + 12 + header.len() + entry.sps_pps.len() + entry.keyframe.len());
        buf.extend_from_slice(KEYFRAME_CACHE_MAGIC);
        buf.extend_from_slice(&header_len.to_le_bytes());
        buf.extend_from_slice(&crc32fast::hash(&header).to_le_bytes());
        buf.extend_from_slice(&content_crc.to_le_bytes());
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&entry.sps_pps);
        buf.extend_from_slice(&entry.keyframe);

        std::fs::write(&self.path, buf)?;
        Ok(())
    }
}

impl KeyframeCacheHeader {
    fn encode(&self) -> Result<Vec<u8>, KeyframeCacheFileError> {
        let hash = self.key.hash.as_bytes();
        let hash_len = u32::try_from(hash.len()).map_err(|_| KeyframeCacheFileError::InvalidFormat("hash too large"))?;

        let mut buf = Vec::with_capacity(20 + hash.len());
        buf.extend_from_slice(&hash_len.to_le_bytes());
        buf.extend_from_slice(hash);
        buf.extend_from_slice(&self.key.width.to_le_bytes());
        buf.extend_from_slice(&self.key.height.to_le_bytes());
        buf.extend_from_slice(&self.key.dpi.to_le_bytes());
        buf.extend_from_slice(&self.sps_pps_len.to_le_bytes());
        Ok(buf)
    }

    fn decode(cursor: &mut KeyframeCacheCursor<'_>) -> Result<Self, KeyframeCacheFileError> {
        let hash_len = cursor.read_u32()? as usize;
        let hash = String::from_utf8(cursor.take(hash_len)?.to_vec())?;
        let width = cursor.read_u32()?;
        let height = cursor.read_u32()?;
        let dpi = cursor.read_u32()?;
        let sps_pps_len = cursor.read_u32()?;

        Ok(Self {
            key: KeyframeCacheKey { hash, width, height, dpi },
            sps_pps_len,
        })
    }
}

struct KeyframeCacheCursor<'a> {
    buf: &'a [u8],
    position: usize,
}

impl<'a> KeyframeCacheCursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, position: 0 }
    }

    fn position(&self) -> usize {
        self.position
    }

    fn remaining(&self) -> usize {
        self.buf.len() - self.position
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], KeyframeCacheFileError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(KeyframeCacheFileError::InvalidFormat("length overflow"))?;
        let bytes = self
            .buf
            .get(self.position..end)
            .ok_or(KeyframeCacheFileError::InvalidFormat("truncated data"))?;
        self.position = end;
        Ok(bytes)
    }

    fn read_u32(&mut self) -> Result<u32, KeyframeCacheFileError> {
        let bytes: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| KeyframeCacheFileError::InvalidFormat("invalid u32"))?;
        Ok(u32::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_cache_file(name: &str) -> KeyframeCacheFile {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        KeyframeCacheFile::new(std::env::temp_dir().join(format!("catplay-keyframe-cache-{name}-{}-{unique}", std::process::id())))
    }

    fn key() -> KeyframeCacheKey {
        KeyframeCacheKey {
            hash: "ui-state-hash".to_string(),
            width: 1920,
            height: 720,
            dpi: 160,
        }
    }

    fn entry() -> KeyframeCacheEntry {
        KeyframeCacheEntry {
            sps_pps: vec![0x01, 0x64, 0x00, 0x1f],
            keyframe: vec![0x65, 0x88, 0x84, 0x21, 0xa0],
        }
    }

    #[test]
    fn stores_and_loads_keyframe_cache_entry() {
        let cache = temp_cache_file("round-trip");
        let key = key();
        let entry = entry();

        cache.store(&key, &entry).unwrap();
        let loaded = cache.load(&key).unwrap();

        assert_eq!(loaded, entry);
        let _ = std::fs::remove_file(cache.path());
    }

    #[test]
    fn returns_out_of_date_for_metadata_mismatch() {
        let cache = temp_cache_file("out-of-date");
        let key = key();
        let mut expected = key.clone();
        expected.width += 1;

        cache.store(&key, &entry()).unwrap();
        let err = cache.load(&expected).unwrap_err();

        assert!(matches!(err, KeyframeCacheFileError::OutOfDate));
        let _ = std::fs::remove_file(cache.path());
    }

    #[test]
    fn rejects_header_crc_mismatch() {
        let cache = temp_cache_file("header-crc");
        let key = key();
        cache.store(&key, &entry()).unwrap();

        let mut bytes = std::fs::read(cache.path()).unwrap();
        let header_start = KEYFRAME_CACHE_MAGIC.len() + 12;
        bytes[header_start] ^= 0xff;
        std::fs::write(cache.path(), bytes).unwrap();

        let err = cache.load(&key).unwrap_err();
        assert!(matches!(err, KeyframeCacheFileError::InvalidFormat("header CRC mismatch")));
        let _ = std::fs::remove_file(cache.path());
    }

    #[test]
    fn rejects_content_crc_mismatch() {
        let cache = temp_cache_file("content-crc");
        let key = key();
        cache.store(&key, &entry()).unwrap();

        let mut bytes = std::fs::read(cache.path()).unwrap();
        let last = bytes.last_mut().unwrap();
        *last ^= 0xff;
        std::fs::write(cache.path(), bytes).unwrap();

        let err = cache.load(&key).unwrap_err();
        assert!(matches!(err, KeyframeCacheFileError::InvalidFormat("content CRC mismatch")));
        let _ = std::fs::remove_file(cache.path());
    }
}
