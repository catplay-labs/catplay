use anyhow::{Context, Result, bail};
use plist::{Dictionary, Uid, Value};

pub(crate) struct KeyedArchive<'a> {
    pub(crate) objects: &'a [Value],
}

impl<'a> KeyedArchive<'a> {
    pub(crate) fn new(root: &'a Value) -> Result<Self> {
        let root = as_dict(root, "archive root")?;
        let objects = root.get("$objects").context("archive has no $objects")?;
        Ok(Self {
            objects: as_array(objects, "$objects")?,
        })
    }

    fn object(&self, uid: Uid) -> Result<&'a Value> {
        self.objects
            .get(uid.get() as usize)
            .with_context(|| format!("invalid object UID {}", uid.get()))
    }

    pub(crate) fn resolve(&self, value: &'a Value) -> Result<&'a Value> {
        match value {
            Value::Uid(uid) => self.object(*uid),
            other => Ok(other),
        }
    }

    pub(crate) fn dict(&self, value: &'a Value, what: &str) -> Result<&'a Dictionary> {
        as_dict(self.resolve(value)?, what)
    }

    pub(crate) fn array(&self, value: &'a Value, what: &str) -> Result<Vec<&'a Value>> {
        let dict = self.dict(value, what)?;
        let values = dict
            .get("NS.objects")
            .with_context(|| format!("{what}: missing NS.objects"))?;
        as_array(values, "NS.objects")?
            .iter()
            .map(|v| self.resolve(v))
            .collect()
    }

    pub(crate) fn children(&self, dict: &'a Dictionary, what: &str) -> Result<Vec<&'a Value>> {
        self.array(
            dict.get("children")
                .with_context(|| format!("{what}: missing children"))?,
            what,
        )
    }

    pub(crate) fn string(&self, value: &'a Value, what: &str) -> Result<&'a str> {
        match self.resolve(value)? {
            Value::String(v) => Ok(v),
            v => bail!("{what}: expected string, got {v:?}"),
        }
    }

    pub(crate) fn integer(&self, value: &'a Value, what: &str) -> Result<i64> {
        match self.resolve(value)? {
            Value::Integer(v) => v.as_signed().context("integer out of range"),
            v => bail!("{what}: expected integer, got {v:?}"),
        }
    }

    pub(crate) fn boolean(&self, value: &'a Value, what: &str) -> Result<bool> {
        match self.resolve(value)? {
            Value::Boolean(v) => Ok(*v),
            v => bail!("{what}: expected bool, got {v:?}"),
        }
    }

    pub(crate) fn nullable_string(&self, value: &'a Value, what: &str) -> Result<Option<&'a str>> {
        match self.resolve(value)? {
            Value::String(v) if v == "$null" => Ok(None),
            Value::String(v) => Ok(Some(v)),
            v => bail!("{what}: expected nullable string, got {v:?}"),
        }
    }

    pub(crate) fn optional_string(&self, dict: &'a Dictionary, key: &str) -> Result<Option<String>> {
        dict.get(key)
            .map(|v| self.nullable_string(v, key).map(|v| v.map(str::to_owned)))
            .transpose()
            .map(Option::flatten)
    }

    pub(crate) fn class_name(&self, value: &'a Value) -> Result<&'a str> {
        let object = self.dict(value, "archived object")?;
        let class = self.dict(object.get("$class").context("object has no $class")?, "$class")?;
        self.string(class.get("$classname").context("class has no $classname")?, "$classname")
    }
}

fn as_dict<'a>(value: &'a Value, what: &str) -> Result<&'a Dictionary> {
    match value {
        Value::Dictionary(v) => Ok(v),
        v => bail!("{what}: expected dictionary, got {v:?}"),
    }
}

fn as_array<'a>(value: &'a Value, what: &str) -> Result<&'a [Value]> {
    match value {
        Value::Array(v) => Ok(v),
        v => bail!("{what}: expected array, got {v:?}"),
    }
}
