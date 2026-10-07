mod archive;
mod cli;
mod disjoin;
mod ir;
mod patches;
pub(crate) mod preview;

#[cfg(test)]
mod tests;

pub(crate) use cli::run_cli;

use self::ir::predicate::FlatPredicateToken;
use self::ir::predicate::{PredicateExpr, PredicateToken};
use self::ir::{Bitfield, Command, EnumField, Field, FieldKind, Lingo, Node, Protocol, Source, StorageType, Token, TransactionIdPolicy};
use anyhow::{Context, Result, bail};
use heck::ToSnakeCase;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Write as _,
};

pub(crate) fn generate_lingoes(protocol: &Protocol) -> Result<Vec<(String, String)>> {
    let mut files = Vec::with_capacity(protocol.lingoes.len() + 1);
    let mut strings = BTreeSet::new();
    for lingo in &protocol.lingoes {
        strings.insert(type_name(&lingo.name));
        let mut out = String::new();
        let mut names = Names::default();
        names.used.insert("LingoMessage".to_owned());
        names.used.insert("LingoRegistry".to_owned());
        writeln!(out, "// @generated from Apple ATS Default.lktspec")?;
        writeln!(out, "// Do not edit manually. Regenerate with catplay_lingo.")?;
        writeln!(out)?;
        writeln!(out, "#[allow(unused_imports)]")?;
        writeln!(out, "use crate::{{__String as String, __Vec as Vec}};")?;
        writeln!(out, "#[allow(unused_imports)]")?;
        writeln!(
            out,
            "use crate::{{FlatPredicateToken, Iap1WireSpec, iap1, iap1_bitfield, iap1_bitflags, iap1_disjoint, iap1_enum, iap1_enum_open, iap1_enum_tokens, iap1_record, iap1_registry, predicate_eval::*}};"
        )?;
        writeln!(out)?;
        writeln!(out, "use super::strings::LingoString;")?;
        writeln!(out)?;
        writeln!(
            out,
            "// Lingo {:#04x}: {}{}",
            lingo.id,
            lingo.name,
            if lingo.deprecated { " [deprecated]" } else { "" }
        )?;
        let mut messages = Vec::with_capacity(lingo.commands.len());
        for command in &lingo.commands {
            let name = emit_command(&mut out, &mut names, lingo, command)?;
            messages.push((command.id, name));
        }
        emit_registry(&mut out, lingo, &messages)?;
        strings.extend(names.strings);
        files.push((format!("lingo_{:#x}.rs", lingo.id), out));
    }
    files.push(("strings.rs".to_owned(), generate_strings(&strings)?));
    Ok(files)
}

#[derive(Default)]
struct Names {
    used: HashSet<String>,
    strings: BTreeSet<String>,
}

impl Names {
    fn unique(&mut self, base: String) -> String {
        let name = unique_name(base, &mut self.used);
        self.record(&name);
        name
    }

    fn record(&mut self, name: &str) {
        self.strings.insert(name.to_owned());
    }
}

fn generate_strings(strings: &BTreeSet<String>) -> Result<String> {
    let mut out = String::from(
        "// @generated from Apple ATS Default.lktspec\n// Do not edit manually. Regenerate with catplay_lingo.\n\nuse catplay_reflect::static_strings;\n\n",
    );
    writeln!(out, "static_strings! {{")?;
    writeln!(out, "    pub struct LingoString(u16) {{")?;
    for (index, name) in strings.iter().enumerate() {
        writeln!(out, "        S{} = {name:?},", index + 1)?;
    }
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(out)
}

fn emit_command(out: &mut String, names: &mut Names, lingo: &Lingo, command: &Command) -> Result<String> {
    let command_ty = names.unique(type_name(&command.name));
    let mut aux = String::new();
    let fields = emit_fields(&command.nodes, &command_ty, names, &mut aux)?;
    write!(out, "{aux}")?;
    writeln!(out, "iap1! {{")?;
    writeln!(out, "    #[iap1(")?;
    writeln!(
        out,
        "        context = ctx, strings = LingoString, lingo = {:#04x}, command = {:#06x},",
        lingo.id, command.id
    )?;
    writeln!(
        out,
        "        source = {},",
        match command.source {
            Source::Accessory => "accessory",
            Source::Device => "device",
        }
    )?;
    writeln!(out, "        response = {},", command.response)?;
    writeln!(out, "        ack = {},", command.ack)?;
    writeln!(out, "        deprecated = {},", command.deprecated)?;
    writeln!(
        out,
        "        transaction_id = {}",
        match command.transaction_id {
            TransactionIdPolicy::Permitted => "permitted",
            TransactionIdPolicy::Prohibited => "prohibited",
            TransactionIdPolicy::Required => "required",
        }
    )?;
    writeln!(out, "    )]")?;
    writeln!(out, "    pub struct {command_ty} {{")?;
    write!(out, "{fields}")?;
    writeln!(out, "    }}\n}}\n")?;
    Ok(command_ty)
}

fn emit_registry(out: &mut String, lingo: &Lingo, messages: &[(u16, String)]) -> Result<()> {
    let mut ids = HashSet::new();
    for (id, _) in messages {
        if !ids.insert(*id) {
            bail!("lingo {:#04x} has duplicate command ID {id:#06x}", lingo.id);
        }
    }
    writeln!(out, "iap1_registry! {{")?;
    writeln!(out, "    {:#04x};", lingo.id)?;
    for (id, name) in messages {
        writeln!(out, "    {name} => {id:#06x},")?;
    }
    writeln!(out, "}}\n")?;
    Ok(())
}

fn emit_fields(nodes: &[Node], owner: &str, names: &mut Names, aux: &mut String) -> Result<String> {
    disjoin::emit(nodes, owner, names, aux)
}

#[cfg(test)]
struct PredicateContext<'a> {
    expression: &'a PredicateExpr,
    parent: Option<&'a PredicateContext<'a>>,
}

fn schema_wire(field: &Field, ty: &str, references: &HashMap<String, String>) -> String {
    let argument = |source: &str| {
        source.parse::<usize>().map_or_else(
            |_| {
                references
                    .get(source)
                    .cloned()
                    .unwrap_or_else(|| field_name(source))
            },
            |value| format!("{value}usize"),
        )
    };
    if ty.starts_with("[u8; ") {
        return "scalar".to_owned();
    }
    if let Some(count) = &field.count {
        return format!("counted({})", argument(count));
    }
    if let Some(length) = &field.length {
        let mode = if ty == "String" {
            "utf8"
        } else if ty == "Vec<u8>" {
            "bytes"
        } else {
            "counted"
        };
        return format!("{mode}({})", argument(length));
    }
    match ty {
        "String" => "raw_string".to_owned(),
        "Vec<u8>" => "raw_bytes".to_owned(),
        value if value.starts_with("Vec<") => "counted_remaining".to_owned(),
        _ => "scalar".to_owned(),
    }
}

#[cfg(test)]
fn flat_predicate(context: &PredicateContext<'_>) -> Result<String> {
    let mut out = String::new();
    let mut first = true;
    visit_context_prefix(context, &mut |token| {
        if !first {
            write!(out, ", ")?;
        }
        first = false;
        match token {
            FlatPredicateToken::Field(name) => write!(out, "Field({name:?})"),
            FlatPredicateToken::AdjustedPayloadLength => write!(out, "AdjustedPayloadLength"),
            FlatPredicateToken::HasTransactionId => write!(out, "HasTransactionId"),
            FlatPredicateToken::Integer(value) => write!(out, "Integer({value})"),
            FlatPredicateToken::Bool(value) => write!(out, "Bool({value})"),
            FlatPredicateToken::Eq => write!(out, "Eq"),
            FlatPredicateToken::Ne => write!(out, "Ne"),
            FlatPredicateToken::Lt => write!(out, "Lt"),
            FlatPredicateToken::Le => write!(out, "Le"),
            FlatPredicateToken::Gt => write!(out, "Gt"),
            FlatPredicateToken::Ge => write!(out, "Ge"),
            FlatPredicateToken::And => write!(out, "And"),
            FlatPredicateToken::Or => write!(out, "Or"),
            FlatPredicateToken::Not => write!(out, "Not"),
        }
    })?;
    Ok(out)
}

fn predicate_expr_rust(expr: &PredicateExpr, references: &HashMap<String, String>) -> String {
    use PredicateExpr as E;
    let operand = |expr: &PredicateExpr| -> String {
        match expr {
            E::Field(PredicateToken::Field(name)) => format!(
                "&{}",
                references
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| field_name(name))
            ),
            E::Field(PredicateToken::AdjustedPayloadLength) => "&ctx.adjusted_payload_length".to_owned(),
            E::Field(PredicateToken::HasTransactionId) => "&ctx.has_transaction_id".to_owned(),
            E::Integer(value) => format!("&{value}u64"),
            E::Bool(value) => format!("&{value}"),
            other => format!("&({})", predicate_expr_rust(other, references)),
        }
    };
    let mut has_length = false;
    let mut has_other_field = false;
    expr.visit_prefix(&mut |token| -> Result<()> {
        match token {
            FlatPredicateToken::AdjustedPayloadLength => has_length = true,
            FlatPredicateToken::Field(_) | FlatPredicateToken::HasTransactionId => has_other_field = true,
            _ => {}
        }
        Ok(())
    })
    .expect("infallible predicate inspection");
    let expression = match expr {
        E::Eq(a, b) | E::Ne(a, b) | E::Lt(a, b) | E::Le(a, b) | E::Gt(a, b) | E::Ge(a, b) => {
            let op = match expr {
                E::Eq(..) => "eq",
                E::Ne(..) => "ne",
                E::Lt(..) => "lt",
                E::Le(..) => "le",
                E::Gt(..) => "gt",
                _ => "ge",
            };
            format!("{op}({}, {})", operand(a), operand(b))
        }
        E::And(a, b) => format!(
            "({}) && ({})",
            predicate_expr_rust(a, references),
            predicate_expr_rust(b, references)
        ),
        E::Or(a, b) => format!(
            "({}) || ({})",
            predicate_expr_rust(a, references),
            predicate_expr_rust(b, references)
        ),
        E::Not(value) => format!("!({})", predicate_expr_rust(value, references)),
        _ => format!("truthy({})", operand(expr)),
    };
    if has_length && !has_other_field {
        format!("ctx.encoding() || ({expression})")
    } else {
        expression
    }
}

pub(super) fn predicate_program_rust(expr: &PredicateExpr, field_slots: &HashMap<String, u8>) -> Result<String> {
    let mut has_length = false;
    let mut has_other_field = false;
    expr.visit_prefix(&mut |token| -> Result<()> {
        match token {
            FlatPredicateToken::AdjustedPayloadLength => has_length = true,
            FlatPredicateToken::Field(_) | FlatPredicateToken::HasTransactionId => has_other_field = true,
            _ => {}
        }
        Ok(())
    })?;
    let mut instructions = Vec::new();
    if has_length && !has_other_field {
        instructions.push("FlatPredicateToken::Or".to_owned());
        instructions.push("FlatPredicateToken::Encoding".to_owned());
    }
    expr.visit_prefix(&mut |token| -> Result<()> {
        use FlatPredicateToken as T;
        let instruction = match token {
            T::Field(name) => format!(
                "FlatPredicateToken::Field({})",
                field_slots
                    .get(name)
                    .with_context(|| format!("predicate field {name:?} has no assigned field slot"))?
            ),
            T::AdjustedPayloadLength => "FlatPredicateToken::AdjustedPayloadLength".to_owned(),
            T::HasTransactionId => "FlatPredicateToken::HasTransactionId".to_owned(),
            T::Integer(value) => format!(
                "FlatPredicateToken::Integer({}u8)",
                u8::try_from(value).context("predicate integer exceeds u8::MAX")?
            ),
            T::Bool(value) => format!("FlatPredicateToken::Bool({value})"),
            T::Eq => "FlatPredicateToken::Eq".to_owned(),
            T::Ne => "FlatPredicateToken::Ne".to_owned(),
            T::Lt => "FlatPredicateToken::Lt".to_owned(),
            T::Le => "FlatPredicateToken::Le".to_owned(),
            T::Gt => "FlatPredicateToken::Gt".to_owned(),
            T::Ge => "FlatPredicateToken::Ge".to_owned(),
            T::And => "FlatPredicateToken::And".to_owned(),
            T::Or => "FlatPredicateToken::Or".to_owned(),
            T::Not => "FlatPredicateToken::Not".to_owned(),
        };
        instructions.push(instruction);
        Ok(())
    })?;
    Ok(format!("[{}]", instructions.join(", ")))
}

#[cfg(test)]
fn visit_context_prefix<E>(
    context: &PredicateContext<'_>,
    emit: &mut impl FnMut(FlatPredicateToken<'_>) -> std::result::Result<(), E>,
) -> std::result::Result<(), E> {
    if let Some(parent) = context.parent {
        emit(FlatPredicateToken::And)?;
        visit_context_prefix(parent, emit)?;
    }
    context.expression.visit_prefix(emit)
}

fn emit_field_type(field: &Field, owner: &str, names: &mut Names, aux: &mut String) -> Result<String> {
    emit_field_type_named(field, format!("{owner}{}", type_name(&field.key)), names, aux)
}

fn emit_field_type_named(field: &Field, base: String, names: &mut Names, aux: &mut String) -> Result<String> {
    match &field.kind {
        FieldKind::Scalar(ty) => {
            if matches!(ty, StorageType::Data | StorageType::LingoIdData | StorageType::CharData)
                && let Some(length) = field
                    .length
                    .as_deref()
                    .and_then(|value| value.parse::<usize>().ok())
            {
                Ok(format!("[u8; {length}]"))
            } else {
                Ok(ty.rust().to_owned())
            }
        }
        FieldKind::Enum(schema) => {
            let ty = names.unique(base);
            emit_enum(aux, &ty, schema, names)?;
            Ok(ty)
        }
        FieldKind::Bitfield(schema) => {
            let ty = names.unique(base);
            emit_bitfield(aux, &ty, schema, names)?;
            Ok(ty)
        }
        FieldKind::Collection(nodes) | FieldKind::Records(nodes) => {
            let ty = names.unique(base);
            let fields = emit_fields(nodes, &ty, names, aux)?;
            writeln!(
                aux,
                "iap1_record! {{\n    strings = LingoString;\n    pub struct {ty} {{\n{fields}    }}\n}}\n"
            )?;
            Ok(format!("Vec<{ty}>"))
        }
        FieldKind::Tokens(tokens) => {
            let ty = names.unique(base);
            emit_tokens(aux, &ty, tokens, names)?;
            Ok(format!("Vec<{ty}>"))
        }
    }
}

fn emit_enum(out: &mut String, ty: &str, schema: &EnumField, names: &mut Names) -> Result<()> {
    let storage = integer_storage(schema.storage_ty)?;
    if schema.items.is_empty() {
        writeln!(out, "/// The specification lists no named values for this enum field.")?;
        writeln!(
            out,
            "#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]\npub struct {ty}(pub {storage});\nimpl crate::__NamedDebug for {ty} {{\n    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{ f.debug_tuple(name).field(self).finish() }}\n}}\n"
        )?;
        emit_raw_newtype_codec(out, ty, storage)?;
        return Ok(());
    }
    let closed = schema.ranges.is_empty();
    if closed {
        writeln!(out, "iap1_enum! {{")?;
        writeln!(out, "    strings = LingoString;")?;
    } else {
        writeln!(out, "iap1_enum_open! {{")?;
        writeln!(out, "    strings = LingoString;")?;
        let ranges = schema
            .ranges
            .iter()
            .map(|range| format!("{}..={}", range.min, range.max))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "    #[iap1_enum_open(storage = {storage}, ranges = [{ranges}])]")?;
        for range in &schema.ranges {
            writeln!(
                out,
                "    /// Permitted range {}..={}: {}",
                range.min,
                range.max,
                clean_doc(&range.description)
            )?;
        }
    }
    if closed {
        writeln!(out, "    pub enum {ty}: {storage} {{")?;
    } else {
        writeln!(out, "    pub enum {ty} {{")?;
    }
    let mut values = HashMap::<i64, String>::new();
    let mut variants = HashSet::new();
    let mut aliases = Vec::new();
    for item in &schema.items {
        if let Some(existing) = values.get(&item.value) {
            let alias = unique_name(type_name(&item.name), &mut variants);
            names.record(&alias);
            aliases.push((alias, existing.clone()));
            continue;
        }
        let name = unique_name(type_name(&item.name), &mut variants);
        names.record(&name);
        doc(out, "        ", &item.description)?;
        if item.deprecated {
            writeln!(out, "        #[deprecated]")?;
        }
        writeln!(out, "        {name} = {},", item.value)?;
        values.insert(item.value, name);
    }
    writeln!(out, "    }}")?;
    writeln!(out, "}}\n")?;
    if !aliases.is_empty() {
        writeln!(out, "impl {ty} {{")?;
        for (alias, target) in aliases {
            writeln!(out, "    pub const {alias}: Self = Self::{target};")?;
        }
        writeln!(out, "}}")?;
    }
    writeln!(out)?;
    Ok(())
}

fn emit_raw_newtype_codec(out: &mut String, ty: &str, storage: &str) -> Result<()> {
    writeln!(out, "impl crate::Iap1Decode for {ty} {{")?;
    writeln!(
        out,
        "    fn decode(reader: &mut crate::Reader<'_>) -> Result<Self, crate::DecodeError> {{"
    )?;
    writeln!(out, "        Ok(Self(<{storage} as crate::Iap1Decode>::decode(reader)?))")?;
    writeln!(out, "    }}\n}}")?;
    writeln!(out, "impl crate::Iap1Encode for {ty} {{")?;
    writeln!(
        out,
        "    fn encode(&self, writer: &mut crate::Writer<'_>) -> Result<(), crate::EncodeError> {{"
    )?;
    writeln!(out, "        crate::Iap1Encode::encode(&self.0, writer)")?;
    writeln!(out, "    }}\n}}")?;
    writeln!(out, "impl crate::PredicateValue for {ty} {{")?;
    writeln!(out, "    fn predicate_integer(&self) -> Option<i128> {{ Some(self.0 as i128) }}")?;
    writeln!(out, "}}\n")?;
    Ok(())
}

fn emit_bitfield(out: &mut String, ty: &str, schema: &Bitfield, names: &mut Names) -> Result<()> {
    let storage = match schema.storage_ty {
        StorageType::Data => {
            let highest = schema
                .bits
                .iter()
                .map(|b| b.mask.leading_zeros())
                .map(|z| 64 - z)
                .chain(schema.groups.iter().map(|g| 64 - g.mask.leading_zeros()))
                .max()
                .unwrap_or(8);
            match highest {
                0..=8 => "u8",
                9..=16 => "u16",
                17..=32 => "u32",
                _ => "u64",
            }
        }
        other => integer_storage(other)?,
    };
    let width = match storage {
        "u8" => 8,
        "u16" => 16,
        "u32" => 32,
        _ => 64,
    };
    for bit in &schema.bits {
        if bit.mask.leading_zeros() < 64 - width {
            bail!("{ty}: bit exceeds storage width");
        }
    }
    for group in &schema.groups {
        if group.mask.leading_zeros() < 64 - width {
            bail!("{ty}: group exceeds storage width");
        }
    }
    let mut occupied = 0u64;
    for mask in schema
        .bits
        .iter()
        .map(|bit| bit.mask)
        .chain(schema.groups.iter().map(|group| group.mask))
    {
        if occupied & mask != 0 {
            bail!("{ty}: bitfield masks overlap");
        }
        occupied |= mask;
    }
    // Data bitfields index bits within successive bytes, unlike numeric BE scalars.
    let byte_indexed = matches!(schema.storage_ty, StorageType::Data);
    if schema.groups.is_empty() {
        emit_flags(out, ty, storage, &schema.bits, byte_indexed, names)?;
        return Ok(());
    }
    if !schema.bits.is_empty() {
        let flags_ty = format!("{ty}Flags");
        names.record(&flags_ty);
        emit_flags(out, &flags_ty, storage, &schema.bits, byte_indexed, names)?;
    }
    writeln!(out, "iap1_bitfield! {{")?;
    writeln!(
        out,
        "    pub struct {ty}: {storage}{} {{",
        if byte_indexed { ", little_endian" } else { "" }
    )?;
    if !schema.bits.is_empty() {
        let mask = schema.bits.iter().fold(0u64, |a, b| a | b.mask);
        writeln!(out, "        #[flags(mask = {mask:#x})]")?;
        writeln!(out, "        pub flags: {ty}Flags,")?;
    }
    for group in &schema.groups {
        let method = field_name(&group.name);
        let group_ty = format!("{ty}{}", type_name(&group.name));
        names.record(&method);
        names.record(&group_ty);
        writeln!(out, "        #[group(mask = {:#x}, shift = {})]", group.mask, group.shift)?;
        doc(out, "        ", &group.description)?;
        writeln!(out, "        pub {method}: {group_ty} {{")?;
        let mut values = HashSet::new();
        for item in &group.items {
            if values.insert(item.value) {
                let variant = bitgroup_variant(&group.items, item);
                names.record(&variant);
                doc(out, "            ", &item.name)?;
                if item.deprecated {
                    writeln!(out, "            #[deprecated]")?;
                }
                writeln!(out, "            {variant} = {},", item.value)?;
            }
        }
        writeln!(out, "        }},")?;
    }
    writeln!(out, "    }}\n}}\n")?;
    Ok(())
}

fn bitgroup_variant(items: &[self::ir::BitGroupItem], wanted: &self::ir::BitGroupItem) -> String {
    let mut names = HashSet::new();
    names.insert("Other".to_owned());
    for item in items {
        let name = unique_name(type_name(&item.name), &mut names);
        if std::ptr::eq(item, wanted) {
            return name;
        }
    }
    unreachable!()
}

fn emit_flags(out: &mut String, ty: &str, storage: &str, bits: &[self::ir::Bit], byte_indexed: bool, pool: &mut Names) -> Result<()> {
    writeln!(
        out,
        "iap1_bitflags! {{\n    strings = LingoString;\n    pub struct {ty}: {storage}{} {{",
        if byte_indexed { ", little_endian" } else { "" }
    )?;
    let mut names = HashSet::new();
    for bit in bits {
        let name = unique_name(const_name(&bit.name), &mut names);
        pool.record(&name);
        doc(out, "        ", &bit.description)?;
        if bit.required {
            writeln!(out, "        /// Required by the specification.")?;
        }
        if bit.deprecated {
            writeln!(out, "        #[deprecated]")?;
        }
        writeln!(out, "        const {name} = 1 << {};", bit.mask.trailing_zeros())?;
    }
    writeln!(out, "    }}\n}}\n")?;
    Ok(())
}

fn emit_tokens(out: &mut String, ty: &str, tokens: &[Token], names: &mut Names) -> Result<()> {
    let mut variants = HashSet::new();
    let mut entries = Vec::new();
    for token in tokens {
        let variant = unique_name(type_name(&token.name), &mut variants);
        names.record(&variant);
        let payload = names.unique(format!("{ty}{variant}"));
        let fields = emit_fields(&token.nodes, &payload, names, out)?;
        writeln!(
            out,
            "iap1_record! {{\n    strings = LingoString;\n    pub struct {payload} {{\n{fields}    }}\n}}\n"
        )?;
        entries.push((
            variant,
            payload,
            u8::try_from(token.fid_type)?,
            u8::try_from(token.fid_subtype)?,
            token.deprecated,
        ));
    }
    writeln!(out, "iap1_enum_tokens! {{")?;
    writeln!(out, "    strings = LingoString;")?;
    writeln!(out, "    pub enum {ty} {{")?;
    for (variant, payload, fid_type, fid_subtype, deprecated) in &entries {
        writeln!(out, "        #[iap1_token(fid_type = {fid_type}, fid_subtype = {fid_subtype})]")?;
        if *deprecated {
            writeln!(out, "        #[deprecated]")?;
        }
        writeln!(out, "        {variant}({payload}),")?;
    }
    writeln!(out, "    }}\n}}\n")?;
    Ok(())
}

fn integer_storage(ty: StorageType) -> Result<&'static str> {
    let rust = ty.rust();
    if ty.bits().is_some() {
        Ok(rust)
    } else {
        bail!("non-integer storage {rust}")
    }
}

fn doc(out: &mut String, indent: &str, value: &str) -> Result<()> {
    writeln!(out, "{indent}/// {}", clean_doc(value))?;
    Ok(())
}

fn clean_doc(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

fn unique_name(base: String, used: &mut HashSet<String>) -> String {
    if used.insert(base.clone()) {
        return base;
    }
    for n in 2.. {
        let candidate = format!("{base}{n}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

fn type_name(value: &str) -> String {
    let mut result = String::new();
    let mut upper = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if upper {
                result.push(ch.to_ascii_uppercase());
            } else {
                result.push(ch);
            }
            upper = false;
        } else {
            upper = true;
        }
    }
    if result.is_empty() || result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert_str(0, "Value");
    }
    result
}

fn field_name(value: &str) -> String {
    let mut result = value.to_snake_case();
    if result.is_empty() || result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert_str(0, "value_");
    }
    match result.as_str() {
        "self" | "super" | "crate" | "Self" => result.push('_'),
        "type" | "match" | "ref" | "mod" | "struct" | "enum" | "fn" | "pub" | "impl" | "where" | "loop" | "move" | "async" | "await"
        | "dyn" | "use" | "in" | "as" | "const" | "static" | "trait" | "return" => result.insert_str(0, "r#"),
        _ => {}
    }
    result
}

fn const_name(value: &str) -> String {
    let mut result = field_name(value).replace("r#", "").to_ascii_uppercase();
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert_str(0, "VALUE_");
    }
    result
}

#[cfg(test)]
mod flat_tests {
    use super::*;

    #[test]
    fn nested_predicates_are_prefixed_with_and() -> Result<()> {
        let outer = super::ir::predicate::parse("a == 1")?;
        let inner = super::ir::predicate::parse("b != 2")?;
        let parent = PredicateContext {
            expression: &outer,
            parent: None,
        };
        let child = PredicateContext {
            expression: &inner,
            parent: Some(&parent),
        };
        assert_eq!(
            flat_predicate(&child)?,
            "And, Eq, Field(\"a\"), Integer(1), Ne, Field(\"b\"), Integer(2)"
        );
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn snake_case_preserves_acronym_boundaries() {
    assert_eq!(field_name("lingoID"), "lingo_id");
    assert_eq!(field_name("USBHostMode"), "usb_host_mode");
    assert_eq!(field_name("RFTuner"), "rf_tuner");
    assert_eq!(field_name("iAP2"), "i_ap2");
    assert_eq!(field_name("type"), "r#type");
    assert_eq!(field_name("123"), "value_123");
    assert_eq!(field_name(""), "value_");
}
