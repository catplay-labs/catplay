#![no_std]

use core::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    Field(&'static str),
    Index(usize),
}

/// One path element.
///
/// Each `Path` lives on the call stack, and `parent` points to the
/// preceding frame.
///
/// `C` is a user-defined context attached to each frame.
/// It defaults to `()`.
#[derive(Debug)]
pub struct Path<'a, C = ()> {
    parent: Option<&'a Path<'a, C>>,
    segment: Option<Segment>,
    context: C,
}

impl<C: Default> Path<'static, C> {
    /// Root without a user-defined context.
    pub fn root() -> Self {
        Self {
            parent: None,
            segment: None,
            context: C::default(),
        }
    }
}

impl<C: Default> Default for Path<'static, C> {
    fn default() -> Self {
        Self::root()
    }
}

impl<C> Path<'static, C> {
    /// Root with a user-defined context.
    pub const fn root_with_context(context: C) -> Self {
        Self {
            parent: None,
            segment: None,
            context,
        }
    }
}

impl<C> Path<'_, C> {
    /// Context attached to this frame.
    #[inline]
    pub const fn context(&self) -> &C {
        &self.context
    }

    /// Mutable access is available only for the current frame.
    #[inline]
    pub const fn context_mut(&mut self) -> &mut C {
        &mut self.context
    }

    #[inline]
    pub const fn segment(&self) -> Option<Segment> {
        self.segment
    }

    #[inline]
    pub const fn parent(&self) -> Option<&Path<'_, C>> {
        self.parent
    }

    /// Creates a child field with its own context.
    #[inline]
    pub fn field_with_context(&self, name: &'static str, context: C) -> Path<'_, C> {
        Path {
            parent: Some(self),
            segment: Some(Segment::Field(name)),
            context,
        }
    }

    /// Creates a child array index with its own context.
    #[inline]
    pub fn index_with_context(&self, index: usize, context: C) -> Path<'_, C> {
        Path {
            parent: Some(self),
            segment: Some(Segment::Index(index)),
            context,
        }
    }

    /// Iterates from the current frame to the root.
    #[inline]
    pub fn ancestors(&self) -> Ancestors<'_, C> {
        Ancestors { next: Some(self) }
    }
}

/// Convenience API for the common case without a context.
impl Path<'_, ()> {
    #[inline]
    pub fn field(&self, name: &'static str) -> Path<'_> {
        self.field_with_context(name, ())
    }

    #[inline]
    pub fn index(&self, index: usize) -> Path<'_> {
        self.index_with_context(index, ())
    }
}

pub struct Ancestors<'a, C> {
    next: Option<&'a Path<'a, C>>,
}

impl<'a, C> Iterator for Ancestors<'a, C> {
    type Item = &'a Path<'a, C>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = current.parent;
        Some(current)
    }
}

fn write_path<C>(path: &Path<'_, C>, out: &mut impl fmt::Write) -> fmt::Result {
    if let Some(parent) = path.parent {
        write_path(parent, out)?;
    }

    match path.segment {
        None => {}

        Some(Segment::Field(name)) => {
            if path.parent.is_some_and(|p| p.segment.is_some()) {
                out.write_char('.')?;
            }

            out.write_str(name)?;
        }

        Some(Segment::Index(index)) => {
            write!(out, "[{index}]")?;
        }
    }

    Ok(())
}

impl<C> fmt::Display for Path<'_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_path(self, f)
    }
}

/// Owned, fixed-capacity snapshot of a path.
///
/// It does not store the context because `C` can be any type and may
/// contain references tied to the call stack.
///
/// Typical use case:
///
/// ```text
/// Path<'_, C> -- error happens --> PathBuf<128>
/// ```
///
/// The path is materialized only when an error occurs.
#[derive(Clone, PartialEq, Eq)]
pub struct PathBuf<const N: usize = 128> {
    bytes: [u8; N],
    len: usize,
    truncated: bool,
}

impl<const N: usize> PathBuf<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
            truncated: false,
        }
    }

    pub fn from_path<C>(path: &Path<'_, C>) -> Self {
        let mut out = Self::new();

        // No heap allocation.
        // Here, fmt::Error only means that the buffer is full.
        let _ = write!(&mut out, "{path}");

        out
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // We only insert complete &str values, so the buffer always
        // contains UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> Default for PathBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Write for PathBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let Some(end) = self.len.checked_add(s.len()) else {
            self.truncated = true;
            return Err(fmt::Error);
        };

        if end > N {
            self.truncated = true;
            return Err(fmt::Error);
        }

        self.bytes[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;

        Ok(())
    }
}

impl<const N: usize> fmt::Display for PathBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())?;

        if self.truncated {
            f.write_str("…")?;
        }

        Ok(())
    }
}

impl<const N: usize> fmt::Debug for PathBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PathBuf")
            .field("path", &self.as_str())
            .field("truncated", &self.truncated)
            .finish()
    }
}

impl<C, const N: usize> From<&Path<'_, C>> for PathBuf<N> {
    fn from(path: &Path<'_, C>) -> Self {
        Self::from_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path() {
        let root = Path::root();
        let a = root.field("a");
        let b = a.field("b");
        let i = b.index(12);
        let c = i.field("c");

        let path = PathBuf::<128>::from_path(&c);

        assert_eq!(path.as_str(), "a.b[12].c");
        assert!(!path.truncated());
    }

    #[test]
    fn nested_arrays() {
        let root = Path::root();

        let a = root.field("a");
        let b = a.field("b");
        let i = b.index(3);
        let c = i.field("c");
        let j = c.index(7);
        let d = j.field("d");

        assert_eq!(PathBuf::<128>::from_path(&d).as_str(), "a.b[3].c[7].d");
    }

    #[test]
    fn custom_context() {
        #[derive(Debug, PartialEq, Eq)]
        enum Context {
            Root,
            FieldId(u16),
            ArrayElement,
        }

        let root = Path::root_with_context(Context::Root);
        let list = root.field_with_context("recents_list", Context::FieldId(1));
        let item = list.index_with_context(4, Context::ArrayElement);
        let index = item.field_with_context("index", Context::FieldId(0));

        assert_eq!(PathBuf::<128>::from_path(&index).as_str(), "recents_list[4].index");
        assert_eq!(index.context(), &Context::FieldId(0));
        assert_eq!(index.parent().unwrap().context(), &Context::ArrayElement);
        assert_eq!(index.parent().unwrap().parent().unwrap().context(), &Context::FieldId(1));
    }

    #[test]
    fn walk_contexts() {
        let root = Path::root_with_context(10);

        let a = root.field_with_context("a", 20);
        let b = a.index_with_context(3, 30);
        let c = b.field_with_context("c", 40);

        let contexts: [i32; 4] = {
            let mut out = [0; 4];

            for (dst, frame) in out.iter_mut().zip(c.ancestors()) {
                *dst = *frame.context();
            }

            out
        };

        assert_eq!(contexts, [40, 30, 20, 10]);
    }

    #[test]
    fn truncation() {
        let root = Path::root();
        let a = root.field("very_long_field_name");

        let path = PathBuf::<8>::from_path(&a);

        assert!(path.truncated());
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DecodeContext {
        Root,
        Field(u16),
        Element(usize),
    }

    fn decode_list_update(path: &Path<'_, DecodeContext>, captured_error_path: &mut PathBuf<128>) {
        let recents_list = path.field_with_context("recents_list", DecodeContext::Field(1));

        decode_recents_list(&recents_list, captured_error_path);
    }

    fn decode_recents_list(path: &Path<'_, DecodeContext>, captured_error_path: &mut PathBuf<128>) {
        for index in 0..3 {
            let item = path.index_with_context(index, DecodeContext::Element(index));

            decode_recent(&item, captured_error_path);
        }
    }

    fn decode_recent(path: &Path<'_, DecodeContext>, captured_error_path: &mut PathBuf<128>) {
        let remote_id = path.field_with_context("remote_id", DecodeContext::Field(1));

        decode_string(&remote_id, captured_error_path);
    }

    fn decode_string(path: &Path<'_, DecodeContext>, captured_error_path: &mut PathBuf<128>) {
        let parent = path.parent().unwrap();

        if parent.context() == &DecodeContext::Element(1) {
            *captured_error_path = PathBuf::from_path(path);

            assert_eq!(path.context(), &DecodeContext::Field(1));
            assert_eq!(parent.segment(), Some(Segment::Index(1)));
        }
    }

    #[test]
    fn recursive_decoder_call_stack() {
        let root = Path::root_with_context(DecodeContext::Root);

        let mut captured_error_path = PathBuf::<128>::new();

        decode_list_update(&root, &mut captured_error_path);

        assert_eq!(captured_error_path.as_str(), "recents_list[1].remote_id");
        assert!(!captured_error_path.truncated());
        assert_eq!(root.context(), &DecodeContext::Root);
        assert_eq!(root.segment(), None);
    }
}
