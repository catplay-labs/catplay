pub(crate) mod predicate;
// Used by duplicate diagnostics and same-type disjoin generation.
#[allow(dead_code)]
pub(crate) mod predicate_analysis;

use self::predicate::PredicateExpr;
use super::archive::KeyedArchive;
use anyhow::{Context, Result, bail};
use plist::{Dictionary, Value};

#[derive(Debug)]
pub(crate) struct Protocol {
    pub lingoes: Vec<Lingo>,
}

#[derive(Debug)]
pub(crate) struct Lingo {
    pub id: u8,
    pub name: String,
    pub deprecated: bool,
    pub commands: Vec<Command>,
}

#[derive(Debug)]
pub(crate) struct Command {
    pub id: u16,
    pub name: String,
    pub source: Source,
    pub response: bool,
    pub ack: bool,
    pub deprecated: bool,
    pub transaction_id: TransactionIdPolicy,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Source {
    Accessory,
    Device,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TransactionIdPolicy {
    Permitted,
    Prohibited,
    Required,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Node {
    Field(Field),
    Predicate { expression: PredicateExpr, nodes: Vec<Node> },
    // ATS templates affect display only and intentionally produce no wire field.
    Template,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Field {
    pub key: String,
    pub description: String,
    pub length: Option<String>,
    pub count: Option<String>,
    pub kind: FieldKind,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FieldKind {
    Scalar(StorageType),
    Enum(EnumField),
    Bitfield(Bitfield),
    Collection(Vec<Node>),
    Records(Vec<Node>),
    Tokens(Vec<Token>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Bool,
    Data,
    String,
    CommandId,
    LongCommandId,
    LingoId,
    LingoIdData,
    CharData,
}

impl StorageType {
    fn parse(value: i64) -> Result<Self> {
        Ok(match value {
            1 => Self::U8,
            2 => Self::U16,
            3 => Self::U32,
            4 => Self::U64,
            5 => Self::I8,
            6 => Self::I16,
            7 => Self::I32,
            8 => Self::I64,
            9 => Self::Bool,
            10 => Self::Data,
            11 => Self::String,
            12 => Self::CommandId,
            13 => Self::LongCommandId,
            14 => Self::LingoId,
            15 => Self::LingoIdData,
            16 => Self::CharData,
            _ => bail!("unknown LKTFieldType {value:#x}"),
        })
    }

    pub fn rust(self) -> &'static str {
        match self {
            Self::U8 | Self::CommandId | Self::LingoId => "u8",
            Self::U16 | Self::LongCommandId => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Bool => "bool",
            Self::String => "String",
            Self::Data | Self::LingoIdData | Self::CharData => "Vec<u8>",
        }
    }

    pub fn bits(self) -> Option<u32> {
        match self {
            Self::U8 | Self::I8 | Self::CommandId | Self::LingoId => Some(8),
            Self::U16 | Self::I16 | Self::LongCommandId => Some(16),
            Self::U32 | Self::I32 => Some(32),
            Self::U64 | Self::I64 => Some(64),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct EnumField {
    pub storage_ty: StorageType,
    pub items: Vec<EnumItem>,
    pub ranges: Vec<EnumRange>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct EnumItem {
    pub value: i64,
    pub name: String,
    pub description: String,
    pub deprecated: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct EnumRange {
    pub min: i64,
    pub max: i64,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Bitfield {
    pub storage_ty: StorageType,
    pub bits: Vec<Bit>,
    pub groups: Vec<BitGroup>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Bit {
    pub mask: u64,
    pub name: String,
    pub description: String,
    pub required: bool,
    pub deprecated: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BitGroup {
    pub mask: u64,
    pub shift: u32,
    pub name: String,
    pub description: String,
    pub items: Vec<BitGroupItem>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BitGroupItem {
    pub value: i64,
    pub name: String,
    pub deprecated: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub name: String,
    pub fid_type: i64,
    pub fid_subtype: i64,
    pub deprecated: bool,
    pub nodes: Vec<Node>,
}

impl Protocol {
    pub fn parse(archive: &KeyedArchive<'_>) -> Result<Self> {
        let object = archive
            .objects
            .iter()
            .find(|v| {
                archive
                    .class_name(v)
                    .is_ok_and(|name| name == "LKTProtocolDefinition")
            })
            .context("LKTProtocolDefinition not found")?;
        let dict = archive.dict(object, "LKTProtocolDefinition")?;
        let lingoes = archive
            .array(required(dict, "lingoes")?, "lingoes")?
            .into_iter()
            .map(|v| Lingo::parse(archive, v))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { lingoes })
    }
}

impl Lingo {
    fn parse<'a>(archive: &KeyedArchive<'a>, value: &'a Value) -> Result<Self> {
        expect_class(archive, value, "LKTLingoDefinition")?;
        let d = archive.dict(value, "lingo")?;
        let id = u8::try_from(archive.integer(required(d, "lingoID")?, "lingoID")?)?;
        let name = archive
            .string(required(d, "lingoName")?, "lingoName")?
            .to_owned();
        let deprecated = archive.boolean(required(d, "isDeprecated")?, "isDeprecated")?;
        let commands = archive
            .array(required(d, "commands")?, "commands")?
            .into_iter()
            .map(|v| Command::parse(archive, v))
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("lingo {id:#04x} {name}"))?;
        Ok(Self {
            id,
            name,
            deprecated,
            commands,
        })
    }
}

impl Command {
    fn parse<'a>(archive: &KeyedArchive<'a>, value: &'a Value) -> Result<Self> {
        expect_class(archive, value, "LKTCommandDefinition")?;
        let d = archive.dict(value, "command")?;
        let id = u16::try_from(archive.integer(required(d, "commandID")?, "commandID")?)?;
        let name = archive.string(required(d, "name")?, "name")?.to_owned();
        let source = match archive.integer(required(d, "source")?, "source")? {
            0 => Source::Accessory,
            1 => Source::Device,
            v => bail!("unknown source {v}"),
        };
        let response = archive.boolean(required(d, "isResponse")?, "isResponse")?;
        let ack = archive.boolean(required(d, "isACK")?, "isACK")?;
        let deprecated = archive.boolean(required(d, "isDeprecated")?, "isDeprecated")?;
        let transaction_id = match archive.integer(required(d, "transactionIDPolicy")?, "transactionIDPolicy")? {
            0 => TransactionIdPolicy::Permitted,
            1 => TransactionIdPolicy::Prohibited,
            2 => TransactionIdPolicy::Required,
            v => bail!("unknown transaction ID policy {v}"),
        };
        let nodes = parse_children(archive, d, "command").with_context(|| format!("command {id:#06x} {name}"))?;
        Ok(Self {
            id,
            name,
            source,
            response,
            ack,
            deprecated,
            transaction_id,
            nodes,
        })
    }
}

fn parse_children<'a>(archive: &KeyedArchive<'a>, d: &'a Dictionary, what: &str) -> Result<Vec<Node>> {
    archive
        .children(d, what)?
        .into_iter()
        .map(|v| parse_node(archive, v))
        .collect()
}

fn parse_node<'a>(archive: &KeyedArchive<'a>, value: &'a Value) -> Result<Node> {
    let class = archive.class_name(value)?;
    let d = archive.dict(value, class)?;
    match class {
        "LKTTemplateDefinition" => Ok(Node::Template),
        "LKTPredicateDefinition" => {
            let source = archive
                .string(required(d, "predicateString")?, "predicateString")?
                .to_owned();
            let expression = predicate::parse(&source).with_context(|| format!("invalid predicate {source:?}"))?;
            predicate::validate_flat_integer_range(&expression)
                .with_context(|| format!("predicate {source:?} cannot be represented by FlatPredicateToken"))?;
            Ok(Node::Predicate {
                expression,
                nodes: parse_children(archive, d, class)?,
            })
        }
        "LKTFieldDefinition" => {
            let storage = StorageType::parse(archive.integer(required(d, "type")?, "type")?)?;
            Ok(Node::Field(parse_field(archive, d, FieldKind::Scalar(storage), true)?))
        }
        "LKTEnumFieldDefinition" => {
            let storage_ty = StorageType::parse(archive.integer(required(d, "type")?, "type")?)?;
            let mut items = Vec::new();
            let mut ranges = Vec::new();
            for child in archive.children(d, class)? {
                let kind = archive.class_name(child)?;
                let cd = archive.dict(child, kind)?;
                match kind {
                    "LKTEnumItemDefinition" => {
                        let description = archive.string(required(cd, "desc")?, "desc")?.to_owned();
                        items.push(EnumItem {
                            value: archive.integer(required(cd, "value")?, "value")?,
                            name: description.clone(),
                            description,
                            deprecated: archive.boolean(required(cd, "isDeprecated")?, "isDeprecated")?,
                        });
                    }
                    "LKTEnumRangeDefinition" => ranges.push(EnumRange {
                        min: archive.integer(required(cd, "min")?, "min")?,
                        max: archive.integer(required(cd, "max")?, "max")?,
                        description: archive.string(required(cd, "desc")?, "desc")?.to_owned(),
                    }),
                    _ => bail!("unexpected {kind} inside enum field"),
                }
            }
            Ok(Node::Field(parse_field(
                archive,
                d,
                FieldKind::Enum(EnumField { storage_ty, items, ranges }),
                false,
            )?))
        }
        "LKTBitfieldDefinition" => {
            let storage_ty = StorageType::parse(archive.integer(required(d, "type")?, "type")?)?;
            let mut bits = Vec::new();
            let mut groups = Vec::new();
            for child in archive.children(d, class)? {
                let kind = archive.class_name(child)?;
                let cd = archive.dict(child, kind)?;
                match kind {
                    "LKTBitDefinition" => {
                        let index = u32::try_from(archive.integer(required(cd, "index")?, "index")?)?;
                        let mask = 1u64.checked_shl(index).context("bit index exceeds u64")?;
                        let description = archive.string(required(cd, "desc")?, "desc")?.to_owned();
                        bits.push(Bit {
                            mask,
                            name: description.clone(),
                            description,
                            required: archive.boolean(required(cd, "required")?, "required")?,
                            deprecated: archive.boolean(required(cd, "isDeprecated")?, "isDeprecated")?,
                        });
                    }
                    "LKTBitgroupDefinition" => groups.push(parse_bitgroup(archive, cd)?),
                    _ => bail!("unexpected {kind} inside bitfield"),
                }
            }
            Ok(Node::Field(parse_field(
                archive,
                d,
                FieldKind::Bitfield(Bitfield { storage_ty, bits, groups }),
                false,
            )?))
        }
        "LKTRecordsDefinition" | "LKTCollectionDefinition" => {
            let nodes = parse_children(archive, d, class)?;
            let kind = if class == "LKTRecordsDefinition" {
                FieldKind::Records(nodes)
            } else {
                FieldKind::Collection(nodes)
            };
            Ok(Node::Field(parse_field(archive, d, kind, false)?))
        }
        "LKTTokensDefinition" => {
            let tokens = archive
                .children(d, class)?
                .into_iter()
                .map(|v| Token::parse(archive, v))
                .collect::<Result<Vec<_>>>()?;
            Ok(Node::Field(parse_field(archive, d, FieldKind::Tokens(tokens), false)?))
        }
        _ => bail!("unsupported schema node {class}"),
    }
}

fn parse_field<'a>(archive: &KeyedArchive<'a>, d: &'a Dictionary, kind: FieldKind, scalar: bool) -> Result<Field> {
    let key = archive.string(required(d, "key")?, "key")?.to_owned();
    let description = archive.string(required(d, "desc")?, "desc")?.to_owned();
    let length = archive.optional_string(d, "length")?;
    let count = archive.optional_string(d, "count")?;
    if scalar {
        for property in ["maxLength", "minValue", "maxValue", "units"] {
            required(d, property).with_context(|| format!("field {key}"))?;
        }
    }
    Ok(Field {
        key,
        description,
        length,
        count,
        kind,
    })
}

fn parse_bitgroup<'a>(archive: &KeyedArchive<'a>, d: &'a Dictionary) -> Result<BitGroup> {
    let first = u32::try_from(archive.integer(required(d, "lower")?, "lower")?)?;
    let second = u32::try_from(archive.integer(required(d, "upper")?, "upper")?)?;
    let shift = first.min(second);
    let upper = first.max(second);
    if upper >= 64 {
        bail!("invalid bitgroup range {first}..={second}");
    }
    let width = upper - shift + 1;
    let mask = (((1u128 << width) - 1) << shift) as u64;
    let name = archive.string(required(d, "key")?, "key")?.to_owned();
    let description = archive.string(required(d, "desc")?, "desc")?.to_owned();
    let mut items = Vec::new();
    for child in archive.children(d, "bitgroup")? {
        expect_class(archive, child, "LKTBitgroupItemDefinition")?;
        let cd = archive.dict(child, "bitgroup item")?;
        items.push(BitGroupItem {
            value: archive.integer(required(cd, "value")?, "value")?,
            name: archive.string(required(cd, "desc")?, "desc")?.to_owned(),
            deprecated: archive.boolean(required(cd, "isDeprecated")?, "isDeprecated")?,
        });
    }
    Ok(BitGroup {
        mask,
        shift,
        name,
        description,
        items,
    })
}

impl Token {
    fn parse<'a>(archive: &KeyedArchive<'a>, value: &'a Value) -> Result<Self> {
        expect_class(archive, value, "LKTTokenDefinition")?;
        let d = archive.dict(value, "token")?;
        Ok(Self {
            name: archive.string(required(d, "name")?, "name")?.to_owned(),
            fid_type: archive.integer(required(d, "fidType")?, "fidType")?,
            fid_subtype: archive.integer(required(d, "fidSubtype")?, "fidSubtype")?,
            deprecated: archive.boolean(required(d, "isDeprecated")?, "isDeprecated")?,
            nodes: parse_children(archive, d, "token")?,
        })
    }
}

fn required<'a>(d: &'a Dictionary, key: &str) -> Result<&'a Value> {
    d.get(key).with_context(|| format!("missing {key}"))
}

fn expect_class(archive: &KeyedArchive<'_>, value: &Value, expected: &str) -> Result<()> {
    let actual = archive.class_name(value)?;
    if actual != expected {
        bail!("expected {expected}, got {actual}");
    }
    Ok(())
}
