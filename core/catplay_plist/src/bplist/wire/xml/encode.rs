use base64::{Engine, engine::general_purpose::STANDARD};

use crate::bplist::wire::{encode::Emit, scalar::ScalarEncoder};

const PROLOGUE: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n";

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub root_element: bool,
    pub indent_char: u8,
    pub indent_count: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            root_element: true,
            indent_char: b'\t',
            indent_count: 1,
        }
    }
}

impl Options {
    pub fn indent(mut self, indent_char: u8, indent_count: usize) -> Self {
        self.indent_char = indent_char;
        self.indent_count = indent_count;
        self
    }

    pub fn root_element(mut self, root_element: bool) -> Self {
        self.root_element = root_element;
        self
    }
}

#[derive(Debug)]
pub enum Error<E> {
    Emit(E),
    UnsupportedUid,
    InvalidState,
}

impl<E: core::fmt::Display> core::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Emit(error) => error.fmt(f),
            Self::UnsupportedUid => f.write_str("XML plist does not support UID"),
            Self::InvalidState => f.write_str("invalid XML writer state"),
        }
    }
}

pub struct Encoder<E: Emit> {
    emit: E,
    options: Options,
    depth: usize,
    wrote_item: bool,
    finished: bool,
}

impl<E: Emit> Encoder<E> {
    pub fn new(mut emit: E, options: Options) -> Result<Self, Error<E::Error>> {
        if options.root_element {
            emit.emit(PROLOGUE).map_err(Error::Emit)?;
        }
        Ok(Self {
            emit,
            options,
            depth: 0,
            wrote_item: false,
            finished: false,
        })
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), Error<E::Error>> {
        self.emit.emit(bytes).map_err(Error::Emit)
    }

    fn prefix(&mut self) -> Result<(), Error<E::Error>> {
        if self.options.indent_count != 0 {
            if self.wrote_item {
                self.write(b"\n")?;
            }
            let padding = [self.options.indent_char; 64];
            let mut count = self
                .depth
                .checked_mul(self.options.indent_count)
                .ok_or(Error::InvalidState)?;
            while count != 0 {
                let n = count.min(padding.len());
                self.write(&padding[..n])?;
                count -= n;
            }
        }
        self.wrote_item = true;
        Ok(())
    }

    fn escaped(&mut self, value: &str) -> Result<(), Error<E::Error>> {
        let bytes = value.as_bytes();
        let mut start = 0;
        for (i, byte) in bytes.iter().enumerate() {
            let escape: &[u8] = match byte {
                b'&' => b"&amp;",
                b'<' => b"&lt;",
                b'>' => b"&gt;",
                b'\"' => b"&quot;",
                b'\'' => b"&apos;",
                _ => continue,
            };
            self.write(&bytes[start..i])?;
            self.write(escape)?;
            start = i + 1;
        }
        self.write(&bytes[start..])
    }

    fn text_element(&mut self, tag: &[u8], value: &str) -> Result<(), Error<E::Error>> {
        self.prefix()?;
        self.write(b"<")?;
        self.write(tag)?;
        self.write(b">")?;
        self.escaped(value)?;
        self.write(b"</")?;
        self.write(tag)?;
        self.write(b">")
    }

    fn empty_element(&mut self, tag: &[u8]) -> Result<(), Error<E::Error>> {
        self.prefix()?;
        self.write(b"<")?;
        self.write(tag)?;
        self.write(b"/>")
    }

    pub fn key(&mut self, value: &str) -> Result<(), Error<E::Error>> {
        self.text_element(b"key", value)
    }
    pub fn date_text(&mut self, value: &str) -> Result<(), Error<E::Error>> {
        self.text_element(b"date", value)
    }

    pub fn begin_array(&mut self) -> Result<(), Error<E::Error>> {
        self.prefix()?;
        self.write(b"<array>")?;
        self.depth = self.depth.checked_add(1).ok_or(Error::InvalidState)?;
        Ok(())
    }
    pub fn end_array(&mut self) -> Result<(), Error<E::Error>> {
        self.end_collection(b"array")
    }
    pub fn begin_dict(&mut self) -> Result<(), Error<E::Error>> {
        self.prefix()?;
        self.write(b"<dict>")?;
        self.depth = self.depth.checked_add(1).ok_or(Error::InvalidState)?;
        Ok(())
    }
    pub fn end_dict(&mut self) -> Result<(), Error<E::Error>> {
        self.end_collection(b"dict")
    }

    fn end_collection(&mut self, tag: &[u8]) -> Result<(), Error<E::Error>> {
        if self.depth == 0 {
            return Err(Error::InvalidState);
        }
        self.depth -= 1;
        self.prefix()?;
        self.write(b"</")?;
        self.write(tag)?;
        self.write(b">")
    }

    pub fn finish(mut self) -> Result<(), Error<E::Error>> {
        if self.depth != 0 || self.finished {
            return Err(Error::InvalidState);
        }
        if self.options.root_element {
            self.write(b"\n</plist>")?;
        }
        self.finished = true;
        Ok(())
    }
}

impl<E: Emit> ScalarEncoder for Encoder<E> {
    type Error = Error<E::Error>;
    type Item = ();

    fn boolean(&mut self, value: bool) -> Result<(), Self::Error> {
        self.empty_element(if value { b"true" } else { b"false" })
    }
    fn integer(&mut self, value: i64) -> Result<(), Self::Error> {
        let mut buf = itoa::Buffer::new();
        self.text_element(b"integer", buf.format(value))
    }
    fn unsigned_integer(&mut self, value: u64) -> Result<(), Self::Error> {
        let mut buf = itoa::Buffer::new();
        self.text_element(b"integer", buf.format(value))
    }
    fn real(&mut self, value: f64) -> Result<(), Self::Error> {
        let mut buf = ryu::Buffer::new();
        self.text_element(b"real", buf.format(value))
    }
    fn string(&mut self, value: &str) -> Result<(), Self::Error> {
        self.text_element(b"string", value)
    }
    fn data(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.prefix()?;
        self.write(b"<data>")?;
        let mut encoded = [0u8; 128];
        for chunk in value.chunks(96) {
            let len = STANDARD
                .encode_slice(chunk, &mut encoded)
                .map_err(|_| Error::InvalidState)?;
            self.write(&encoded[..len])?;
        }
        self.write(b"</data>")
    }
    fn uid(&mut self, _: u64) -> Result<(), Self::Error> {
        Err(Error::UnsupportedUid)
    }
}
