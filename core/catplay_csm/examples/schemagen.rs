use anyhow::{Context, Result, bail};
use heck::{ToSnakeCase, ToUpperCamelCase};
use plist::{Dictionary, Value};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
struct TypeDef {
    name: String,
    id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnumValue {
    value: i64,
    description: String,
}

#[derive(Debug, Clone)]
struct Param {
    id: u16,
    name: String,
    ty: TypeDef,
    count: u8,
    note: Option<String>,
    enum_name: Option<String>,
    enum_values: Vec<EnumValue>,
    children: Vec<Param>,
}

#[derive(Debug, Clone)]
struct Message {
    id: u16,
    name: String,
    #[allow(unused)]
    source: i64,
    params: Vec<Param>,
}

struct Archive {
    objects: Vec<Value>,
    root: usize,
}

impl Archive {
    fn open(path: &Path) -> Result<Self> {
        let root = Value::from_file(path).with_context(|| format!("reading {}", path.display()))?;
        let dict = root
            .as_dictionary()
            .context("archive root is not a dictionary")?;
        let objects = dict
            .get("$objects")
            .and_then(Value::as_array)
            .context("missing $objects")?
            .clone();
        let top = dict
            .get("$top")
            .and_then(Value::as_dictionary)
            .context("missing $top")?;
        let root_uid = top
            .get("root")
            .and_then(Value::as_uid)
            .context("missing $top.root UID")?
            .get() as usize;
        if root_uid >= objects.len() {
            bail!("$top.root UID out of range");
        }
        Ok(Self { objects, root: root_uid })
    }

    fn deref<'a>(&'a self, value: &'a Value) -> Result<&'a Value> {
        match value.as_uid() {
            Some(uid) => self
                .objects
                .get(uid.get() as usize)
                .context("UID out of range"),
            None => Ok(value),
        }
    }

    fn dict<'a>(&'a self, value: &'a Value) -> Result<&'a Dictionary> {
        self.deref(value)?
            .as_dictionary()
            .context("expected dictionary")
    }

    fn list<'a>(&'a self, value: &'a Value) -> Result<Vec<&'a Value>> {
        let value = self.deref(value)?;
        if let Some(array) = value.as_array() {
            return array.iter().map(|v| self.deref(v)).collect();
        }
        if let Some(dict) = value.as_dictionary() {
            if let Some(objects) = dict.get("NS.objects") {
                let objects = self.deref(objects)?;
                let array = objects.as_array().context("NS.objects is not an array")?;
                return array.iter().map(|v| self.deref(v)).collect();
            }
        }
        Ok(Vec::new())
    }

    fn string(&self, value: &Value) -> Result<String> {
        self.deref(value)?
            .as_string()
            .map(str::to_owned)
            .context("expected string")
    }

    fn integer(&self, value: &Value) -> Result<i64> {
        let value = self.deref(value)?;
        if let Some(v) = value.as_signed_integer() {
            return Ok(v);
        }
        if let Some(v) = value.as_unsigned_integer() {
            return i64::try_from(v).context("integer does not fit i64");
        }
        bail!("expected integer")
    }

    fn opt_string(&self, dict: &Dictionary, key: &str) -> Result<Option<String>> {
        match dict.get(key) {
            Some(v) => Ok(Some(self.string(v)?)),
            None => Ok(None),
        }
    }

    fn parse_type(&self, value: &Value) -> Result<TypeDef> {
        let dict = self.dict(value)?;
        let name = match dict.get("typeString") {
            Some(v) => self.string(v)?,
            None => String::new(),
        };
        let id = match dict.get("typeInEnum") {
            Some(v) => self.integer(v)?,
            None => -1,
        };
        Ok(TypeDef { name, id })
    }

    fn parse_param(&self, value: &Value) -> Result<Param> {
        let dict = self.dict(value)?;
        let id = u16::try_from(self.integer(dict.get("identifier").context("parameter identifier")?)?)
            .context("parameter identifier does not fit u16")?;
        let name = self.string(dict.get("name").context("parameter name")?)?;
        let ty = self.parse_type(dict.get("type").context("parameter type")?)?;
        let count = u8::try_from(
            self.integer(
                dict.get("countExpressionEnum")
                    .context("countExpressionEnum")?,
            )?,
        )
        .context("countExpressionEnum does not fit u8")?;
        let note = self.opt_string(dict, "note")?;
        let enum_name = self.opt_string(dict, "enumName")?;

        let mut enum_values = Vec::new();
        if let Some(v) = dict.get("enumDefinitions") {
            for entry in self.list(v)? {
                let e = self.dict(entry)?;
                enum_values.push(EnumValue {
                    value: self.integer(e.get("value").context("enum value")?)?,
                    description: self.opt_string(e, "valueDescription")?.unwrap_or_default(),
                });
            }
        }

        let mut children = Vec::new();
        if let Some(v) = dict.get("parameterDefinitions") {
            for child in self.list(v)? {
                children.push(self.parse_param(child)?);
            }
        }

        Ok(Param {
            id,
            name,
            ty,
            count,
            note,
            enum_name,
            enum_values,
            children,
        })
    }

    fn messages(&self) -> Result<Vec<Message>> {
        let root = self.objects.get(self.root).context("missing root object")?;
        let mut out = Vec::new();
        for value in self.list(root)? {
            let dict = self.dict(value)?;
            if !dict.contains_key("identifier") || !dict.contains_key("source") || !dict.contains_key("parameterDefinitions") {
                continue;
            }
            let id = u16::try_from(self.integer(dict.get("identifier").unwrap())?).context("message identifier does not fit u16")?;
            let name = self.string(dict.get("name").context("message name")?)?;
            let source = self.integer(dict.get("source").unwrap())?;
            let mut params = Vec::new();
            for p in self.list(dict.get("parameterDefinitions").unwrap())? {
                params.push(self.parse_param(p)?);
            }
            out.push(Message { id, name, source, params });
        }
        out.sort_by_key(|m| m.id);
        Ok(out)
    }
}

#[derive(Debug, Clone)]
enum Packed {
    Dynamic,
    Fixed(Vec<usize>),
}

#[derive(Debug, Clone)]
struct FieldType {
    base: String,
    packed: Option<Packed>,
    // `none` already models presence itself; do not turn optional none into Option<CsmFlag>.
    force_single_count: bool,
}

fn duplicated_type_names(messages: &[Message]) -> HashSet<String> {
    let mut counts = HashMap::<String, usize>::new();
    let mut enum_definitions = HashMap::<String, Vec<Vec<EnumValue>>>::new();

    for message in messages {
        *counts.entry(rust_type_name(&message.name)).or_default() += 1;
        count_nested_type_names(&message.params, &mut counts, &mut enum_definitions);
    }

    for (name, definitions) in enum_definitions {
        *counts.entry(name).or_default() += definitions.len();
    }

    counts
        .into_iter()
        .filter_map(|(name, count)| (count > 1).then_some(name))
        .collect()
}

fn count_nested_type_names(
    params: &[Param],
    counts: &mut HashMap<String, usize>,
    enum_definitions: &mut HashMap<String, Vec<Vec<EnumValue>>>,
) {
    for param in params {
        match param.ty.name.as_str() {
            "enum" | "enum[0+]" => {
                let desired = param.enum_name.as_deref().unwrap_or(&param.name);
                let definitions = enum_definitions.entry(rust_type_name(desired)).or_default();
                if !definitions.contains(&param.enum_values) {
                    definitions.push(param.enum_values.clone());
                }
            }
            "group" => {
                *counts.entry(rust_type_name(&param.name)).or_default() += 1;
            }
            _ => {}
        }
        count_nested_type_names(&param.children, counts, enum_definitions);
    }
}

struct Generator {
    definitions: Vec<String>,
    names: HashSet<String>,
    strings: BTreeSet<String>,
    name_counts: HashMap<String, usize>,
    duplicate_type_names: HashSet<String>,
    enum_types: HashMap<String, Vec<(Vec<EnumValue>, String)>>,
    fixed_array_re: Regex,
}

impl Generator {
    fn new(messages: &[Message]) -> Self {
        Self {
            definitions: Vec::new(),
            names: HashSet::new(),
            strings: BTreeSet::new(),
            name_counts: HashMap::new(),
            duplicate_type_names: duplicated_type_names(messages),
            enum_types: HashMap::new(),
            fixed_array_re: Regex::new(r"^uint8((?:\[[0-9]+\])+)$").unwrap(),
        }
    }

    fn unique_type_from_base(&mut self, mut base: String) -> String {
        if base.is_empty() {
            base = "GeneratedType".into();
        }
        if self.names.insert(base.clone()) {
            return base;
        }
        let counter = self.name_counts.entry(base.clone()).or_insert(1);
        loop {
            *counter += 1;
            let candidate = format!("{base}{counter}");
            if self.names.insert(candidate.clone()) {
                return candidate;
            }
        }
    }

    fn nested_enum_type_base(&self, desired: &str, owner: &str) -> String {
        let base = rust_type_name(desired);
        if self.duplicate_type_names.contains(&base) {
            format!("{}{base}", rust_type_name(owner))
        } else {
            base
        }
    }

    fn unique_struct_type(&mut self, apple_name: &str) -> Result<String> {
        if !is_rust_identifier(apple_name) || is_keyword(apple_name) {
            bail!("Apple struct name is not a Rust identifier: {apple_name:?}");
        }
        Ok(self.unique_type_from_base(apple_name.to_owned()))
    }

    fn enum_type(&mut self, param: &Param, owner: &str) -> Result<String> {
        if param.enum_values.is_empty() {
            bail!("{}: enum without values", param.name);
        }
        if param
            .enum_values
            .iter()
            .any(|v| !(0..=255).contains(&v.value))
        {
            bail!("{}: enum_type! currently requires u8 values", param.name);
        }
        let desired = param.enum_name.as_deref().unwrap_or(&param.name);
        let enum_base = self.nested_enum_type_base(desired, owner);
        if let Some(name) = self.enum_types.get(&enum_base).and_then(|definitions| {
            definitions
                .iter()
                .find(|(values, _)| values == &param.enum_values)
                .map(|(_, name)| name.clone())
        }) {
            return Ok(name);
        }
        let name = self.unique_type_from_base(enum_base.clone());
        let mut s = String::new();
        s.push_str("enum_type! {\n");
        s.push_str(&format!("    pub enum {name} {{\n"));
        let mut used = HashSet::new();
        for entry in &param.enum_values {
            let mut variant = rust_type_name(&entry.description);
            if variant.is_empty() || variant.len() > 64 {
                variant = format!("Value{}", entry.value);
            }
            if !used.insert(variant.clone()) {
                variant = format!("{variant}Value{}", entry.value);
            }
            if !entry.description.is_empty() {
                for line in entry.description.lines() {
                    s.push_str(&format!("        /// {}\n", line.trim()));
                }
            }
            s.push_str(&format!("        {variant} = {},\n", entry.value));
        }
        s.push_str("    }\n}\n");
        self.definitions.push(format!("// owner: {owner}\n{s}"));
        self.enum_types
            .entry(enum_base)
            .or_default()
            .push((param.enum_values.clone(), name.clone()));
        Ok(name)
    }

    fn group_type(&mut self, param: &Param, owner: &str) -> Result<String> {
        if !is_rust_identifier(&param.name) || is_keyword(&param.name) {
            bail!("Apple struct name is not a Rust identifier: {:?}", param.name);
        }
        let raw_base = rust_type_name(&param.name);
        let base = if self.duplicate_type_names.contains(&raw_base) {
            format!("{owner}{}", param.name)
        } else {
            param.name.clone()
        };
        let name = self.unique_type_from_base(base);
        self.strings.insert(name.clone());
        let fields = self.fields(&param.children, &name)?;
        let mut s = String::new();
        s.push_str("group_type! {\n");
        s.push_str("    strings = Iap2String;\n");
        s.push_str(&format!("    pub struct {name} {{\n"));
        s.push_str(&fields);
        s.push_str("    }\n}\n");
        self.definitions.push(format!("// owner: {owner}\n{s}"));
        Ok(name)
    }

    fn field_type(&mut self, param: &Param, owner: &str) -> Result<FieldType> {
        let token = param.ty.name.as_str();
        let scalar = match token {
            "int8" => Some("i8"),
            "int16" => Some("i16"),
            "int32" => Some("i32"),
            "int64" => Some("i64"),
            "uint8" => Some("u8"),
            "uint16" => Some("u16"),
            "uint32" => Some("u32"),
            "uint64" => Some("u64"),
            "bool" => Some("bool"),
            "utf8" => Some("CsmString"),
            "blob" => Some("CsmByteArray"),
            "secs32" => Some("CsmSeconds32"),
            "secs64" => Some("CsmSeconds64"),
            "msecs16" => Some("CsmMilliseconds16"),
            "msecs32" => Some("CsmMilliseconds32"),
            "rat16" => Some("CsmRat16"),
            "rat32" => Some("CsmRat32"),
            "urat16" => Some("CsmURat16"),
            "urat32" => Some("CsmURat32"),
            _ => None,
        };
        if let Some(base) = scalar {
            return Ok(FieldType {
                base: base.into(),
                packed: None,
                force_single_count: false,
            });
        }

        match token {
            "none" => Ok(FieldType {
                base: "CsmFlag".into(),
                packed: None,
                force_single_count: param.count == 1 || param.count == 3,
            }),
            "enum" => Ok(FieldType {
                base: self.enum_type(param, owner)?,
                packed: None,
                force_single_count: false,
            }),
            "enum[0+]" => Ok(FieldType {
                base: self.enum_type(param, owner)?,
                packed: Some(Packed::Dynamic),
                force_single_count: false,
            }),
            "group" => Ok(FieldType {
                base: self.group_type(param, owner)?,
                packed: None,
                force_single_count: false,
            }),
            "uint8[0+]" => Ok(FieldType {
                base: "u8".into(),
                packed: Some(Packed::Dynamic),
                force_single_count: false,
            }),
            "uint16[]" | "uint16[0+]" => Ok(FieldType {
                base: "u16".into(),
                packed: Some(Packed::Dynamic),
                force_single_count: false,
            }),
            "uint64[0+]" => Ok(FieldType {
                base: "u64".into(),
                packed: Some(Packed::Dynamic),
                force_single_count: false,
            }),
            _ => {
                if let Some(caps) = self.fixed_array_re.captures(token) {
                    let dims_text = caps.get(1).unwrap().as_str();
                    let dims_re = Regex::new(r"\[([0-9]+)\]").unwrap();
                    let dims = dims_re
                        .captures_iter(dims_text)
                        .map(|c| c[1].parse::<usize>())
                        .collect::<std::result::Result<Vec<_>, _>>()?;
                    if dims.is_empty() || dims.iter().any(|&n| n == 0) {
                        bail!("{}: invalid fixed packed dimensions", param.name);
                    }
                    return Ok(FieldType {
                        base: "u8".into(),
                        packed: Some(Packed::Fixed(dims)),
                        force_single_count: false,
                    });
                }
                bail!(
                    "{}: unsupported Apple field type {:?} (typeInEnum={})",
                    param.name,
                    token,
                    param.ty.id
                )
            }
        }
    }

    fn fields(&mut self, params: &[Param], owner: &str) -> Result<String> {
        let mut s = String::new();
        let mut ids = HashSet::new();
        let mut fields = HashSet::new();
        for p in params {
            if !ids.insert(p.id) {
                bail!("{owner}: duplicate parameter id {}", p.id);
            }
            let mut field = rust_field_name(&p.name);
            if !fields.insert(field.clone()) {
                field = format!("{field}_id_{}", p.id);
            }
            self.strings.insert(field.clone());
            let ft = self.field_type(p, owner)?;
            let count = if ft.force_single_count {
                "one"
            } else {
                match p.count {
                    0 => "zero_or_more",
                    1 => "one",
                    2 => "one_or_more",
                    3 => "optional",
                    other => bail!("{owner}/{}: unknown countExpressionEnum {other}", p.name),
                }
            };

            s.push_str(&format!("        #[csm_id({})]\n", p.id));
            s.push_str(&format!("        #[csm_count({count})]\n"));
            match &ft.packed {
                Some(Packed::Dynamic) => s.push_str("        #[csm_packed(dynamic)]\n"),
                Some(Packed::Fixed(dims)) => {
                    let dims = dims
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    s.push_str(&format!("        #[csm_packed({dims})]\n"));
                }
                None => {}
            }
            /*s.push_str(&format!(
                "        /// Apple: {}; type={}; count={}.\n",
                p.name,
                p.ty.name,
                apple_count_name(p.count)
            ));*/
            if let Some(note) = &p.note
                && note != "$null"
            {
                for line in note.lines() {
                    s.push_str(&format!("        /// {}\n", line.trim()));
                }
            }
            s.push_str(&format!("        pub {field}: {},\n", ft.base));
        }
        Ok(s)
    }

    fn generate(&mut self, messages: &[Message], archive_name: &str, hash: &str) -> Result<String> {
        let mut packets = String::new();
        let mut registrations = String::new();

        for m in messages {
            let name = self.unique_struct_type(&m.name)?;
            self.strings.insert(name.clone());
            let fields = self.fields(&m.params, &name)?;
            packets.push_str("packet_type! {\n");
            packets.push_str("    @no_register\n");
            packets.push_str("    strings = Iap2String;\n");
            /*packets.push_str(&format!(
                "    /// Apple: {}; source={}.\n",
                m.name, m.source
            ));*/
            packets.push_str(&format!("    pub struct {name}: 0x{:04X} {{\n", m.id));
            packets.push_str(&fields);
            packets.push_str("    }\n}\n\n");
            registrations.push_str(&format!("    0x{:04X} => {name},\n", m.id));
        }

        if !registrations.is_empty() {
            packets.push_str("register_packet_family! {\n");
            packets.push_str(&registrations);
            packets.push_str("}\n");
        }

        let mut out = String::new();
        out.push_str(&format!("// Generated from {archive_name}\n"));
        out.push_str(&format!("// SHA-256: {hash}\n"));
        out.push_str("// Generated by examples/schemagen.rs. Do not hand-edit.\n");
        out.push_str("// Required fields remain permissive at runtime; missing values keep Default.\n\n");
        out.push_str("#![allow(non_camel_case_types)]\n\n");
        out.push_str("use crate::{decoder::*, enum_type, group_type, packet_type, register_packet_family};\n\n");
        out.push_str("#[cfg(feature = \"project\")]\nuse super::iap2_strings::Iap2String;\n\n");
        for definition in &self.definitions {
            out.push_str(definition);
            out.push('\n');
        }
        out.push_str(&packets);
        Ok(out)
    }

    fn generate_strings(&self, archive_name: &str, hash: &str) -> String {
        let mut out = format!(
            "// Generated from {archive_name}\n// SHA-256: {hash}\n// Generated by examples/schemagen.rs. Do not hand-edit.\n\nuse catplay_reflect::static_strings;\n\n"
        );
        out.push_str("static_strings! {\n    pub struct Iap2String(u16) {\n");
        for (index, name) in self.strings.iter().enumerate() {
            out.push_str(&format!("        S{} = {name:?},\n", index + 1));
        }
        out.push_str("    }\n}\n");
        out
    }
}

#[allow(unused)]
fn apple_count_name(v: u8) -> &'static str {
    match v {
        0 => "0+",
        1 => "1",
        2 => "1+",
        3 => "0..1",
        _ => "unknown",
    }
}

fn clean_words(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>()
}

fn rust_type_name(input: &str) -> String {
    let mut s = clean_words(input).to_upper_camel_case();
    if s.is_empty() {
        return s;
    }
    if s.as_bytes()[0].is_ascii_digit() {
        s.insert_str(0, "Value");
    }
    if is_keyword(&s) {
        s.push_str("Value");
    }
    s
}

fn rust_field_name(input: &str) -> String {
    let input = input
        .replace("iAP2", " iAP2 ")
        .replace("IAP2", " IAP2 ")
        .replace("iap2", " iap2 ")
        .replace("WiFi", " WiFi ")
        .replace("WIFI", " WIFI ")
        .replace("wifi", " wifi ");
    let mut s = clean_words(&input)
        .to_snake_case()
        .replace("i_ap2", "iap2")
        .replace("wi_fi", "wifi");
    if s.is_empty() {
        s.push_str("value");
    }
    if s.as_bytes()[0].is_ascii_digit() {
        s.insert_str(0, "value_");
    }
    if is_keyword(&s) {
        s.push('_');
    }
    s
}

fn is_rust_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('_' | 'a'..='z' | 'A'..='Z')) && chars.all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

fn parse_u16(s: &str) -> Result<u16> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Ok(u16::from_str_radix(hex, 16)?)
    } else {
        Ok(s.parse()?)
    }
}

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .context("usage: i2mspec_codegen <archive> [-o output.rs] [--message 0x1234 ...]")?;
    let (archive, mut output) = if input == "--regenerate" {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        (
            crate_dir.join("examples/iap2messages-internal.i2mspecarchive"),
            Some(crate_dir.join("src/msg/iap2.rs")),
        )
    } else {
        (PathBuf::from(input), None)
    };
    let mut wanted = HashSet::<u16>::new();

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "-o" | "--output" => {
                output = Some(PathBuf::from(args.next().context("missing output path")?));
            }
            "--message" => {
                let value = args.next().context("missing --message value")?;
                wanted.insert(parse_u16(&value.to_string_lossy())?);
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    let bytes = fs::read(&archive)?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let parsed = Archive::open(&archive)?;
    let mut messages = parsed.messages()?;
    let mut generator = Generator::new(&messages);
    if !wanted.is_empty() {
        let available = messages.iter().map(|m| m.id).collect::<HashSet<_>>();
        for id in &wanted {
            if !available.contains(id) {
                bail!("message 0x{id:04X} not found");
            }
        }
        messages.retain(|m| wanted.contains(&m.id));
    }

    let archive_name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("i2mspecarchive");
    let source = generator.generate(&messages, archive_name, &hash)?;
    let strings_source = generator.generate_strings(archive_name, &hash);

    if let Some(path) = output {
        fs::write(&path, source)?;
        fs::write(path.with_file_name("iap2_strings.rs"), strings_source)?;
    } else {
        print!("{source}");
    }
    eprintln!("generated {} messages", messages.len());
    Ok(())
}
