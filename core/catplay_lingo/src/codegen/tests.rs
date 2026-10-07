use super::{archive::KeyedArchive, generate_lingoes, ir::Protocol};
use anyhow::Result;
use plist::Value;
use std::io::Cursor;

#[test]
fn default_spec_keeps_structured_nodes() -> Result<()> {
    let root = Value::from_reader(Cursor::new(include_bytes!("../Default.lktspec")))?;
    let protocol = Protocol::parse(&KeyedArchive::new(&root)?)?;
    assert_eq!(
        protocol
            .lingoes
            .iter()
            .map(|l| l.commands.len())
            .sum::<usize>(),
        338
    );
    let files = generate_lingoes(&protocol)?;
    assert_eq!(files.len(), 16);
    assert_eq!(files[13].0, "lingo_0xd.rs");
    assert_eq!(files[15].0, "strings.rs");
    for (name, source) in &files {
        if name == "strings.rs" {
            continue;
        }
        assert_eq!(
            source
                .lines()
                .filter(|line| line.starts_with("// Lingo 0x"))
                .count(),
            1,
            "{name}"
        );
    }
    let source = files
        .into_iter()
        .map(|(_, source)| source)
        .collect::<String>();
    assert_eq!(source.matches("iap1! {").count(), 338);
    assert_eq!(
        source.matches("strings = LingoString").count(),
        source.matches("iap1! {").count()
            + source.matches("iap1_record! {").count()
            + source.matches("iap1_enum! {").count()
            + source.matches("iap1_disjoint! {").count()
            + source.matches("iap1_enum_open! {").count()
            + source.matches("iap1_enum_tokens! {").count()
            + source.matches("iap1_bitflags! {").count()
    );
    assert_eq!(source.matches("use super::strings::LingoString;").count(), 15);
    assert!(!source.contains("#[iap1_field("));
    assert!(source.matches("fields {").count() >= 338);
    assert_eq!(source.matches("fields {").count(), source.matches("steps {").count());
    assert_eq!(source.matches("iap1_registry! {").count(), 15);
    assert_eq!(
            source
                .matches("FlatPredicateToken, Iap1WireSpec, iap1, iap1_bitfield, iap1_bitflags, iap1_disjoint, iap1_enum, iap1_enum_open, iap1_enum_tokens, iap1_record,")
                .count(),
            15
        );
    assert!(source.contains("iap1_bitflags!"));
    assert!(source.contains("iap1_bitfield!"));
    assert!(source.contains("pub authentication_control_bits:"));
    assert!(source.contains("pub enum IPodAckCommandResult: u8"));
    assert!(source.contains("iap1_enum! {"));
    assert!(source.contains("iap1_enum_open! {"));
    assert!(source.contains("iap1_enum_tokens! {"));
    assert!(source.contains("#[iap1_token(fid_type = 0, fid_subtype = 0)]"));
    assert!(source.contains("pub struct BeginRecord"));
    assert!(source.contains("pub enum SetFIDTokenValuesTokens"));
    assert!(!source.contains("pub struct Lingo0"));
    assert!(!source.contains("pub enum Lingo0"));
    assert!(!source.contains("flat:"));
    assert!(source.contains(
        "predicate: { expr: eq(&command_result, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] }"
    ));
    assert!(source.contains("optional pub maximum_pending_wait: u32"));
    assert!(!source.contains("source: \"commandResult == 0x06\""));
    Ok(())
}

#[test]
fn string_pool_contains_generated_names_once() -> Result<()> {
    let root = Value::from_reader(Cursor::new(include_bytes!("../Default.lktspec")))?;
    let mut protocol = Protocol::parse(&KeyedArchive::new(&root)?)?;
    super::patches::apply(&mut protocol)?;
    let files = generate_lingoes(&protocol)?;
    let pool = &files.last().unwrap().1;
    assert!(pool.contains("pub struct LingoString(u16)"));
    assert!(pool.contains("= \"BeginRecord\","));
    assert!(pool.contains("= \"authentication_control_bits\","));
    assert!(pool.contains("= \"accInfoType\","));
    assert!(!pool.contains("= \"acc_info_type\","));
    assert!(pool.contains("= \"AccessoryEqualizerOptions\","));
    assert!(pool.contains("= \"General\","));
    assert!(pool.contains("= \"USBHostMode\","));
    let values: Vec<_> = pool
        .lines()
        .filter_map(|line| line.split_once(" = ").map(|(_, value)| value))
        .collect();
    let checked_in_values: Vec<_> = include_str!("../lingos/strings.rs")
        .lines()
        .filter_map(|line| line.split_once(" = ").map(|(_, value)| value))
        .collect();
    assert_eq!(values.len(), checked_in_values.len());
    for (index, (generated, checked_in)) in values.iter().zip(checked_in_values).enumerate() {
        assert_eq!(generated, &checked_in, "pool entry {index}");
    }
    assert_eq!(
        values.len(),
        values
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
    );
    Ok(())
}

#[test]
fn type_name_fallback_checks_explicit_and_generated_names() {
    let mut names = super::Names::default();
    assert_eq!(names.unique("Child".into()), "Child");
    assert_eq!(names.unique("Child2".into()), "Child2");
    assert_eq!(names.unique("Child".into()), "Child3");
}

#[test]
fn length_predicates_generate_encode_bypass_and_selected_storage() -> Result<()> {
    let root = Value::from_reader(Cursor::new(include_bytes!("../Default.lktspec")))?;
    let protocol = Protocol::parse(&KeyedArchive::new(&root)?)?;
    let files = generate_lingoes(&protocol)?;
    let general = &files
        .iter()
        .find(|(name, _)| name == "lingo_0x0.rs")
        .unwrap()
        .1;
    assert!(general.contains("length_optional pub challenge:"));
    assert!(general.contains("ctx.encoding() || (eq(&ctx.adjusted_payload_length, &17u64))"));
    let remote = &files
        .iter()
        .find(|(name, _)| name == "lingo_0x2.rs")
        .unwrap()
        .1;
    assert!(remote.contains("length_optional pub button_states1:"));
    assert!(remote.contains("ctx.encoding() || (gt(&ctx.adjusted_payload_length, &1u64))"));
    Ok(())
}
