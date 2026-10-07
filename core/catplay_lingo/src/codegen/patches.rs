//! Corrections to the IR loaded from the binary ATS schema; the source archive
//! remains unchanged. Keep corrections here so regeneration preserves them.
use super::ir::{Bit, Bitfield, Field, FieldKind, Node, Protocol, StorageType, predicate};
use anyhow::{Context, Result, ensure};

pub(crate) fn apply(protocol: &mut Protocol) -> Result<()> {
    patch_rf_certifications(protocol)?;
    patch_lingo_option_descriptions(protocol)?;
    patch_preference_descriptions(protocol)?;
    patch_user_data_gender_description(protocol)
}

fn patch_rf_certifications(protocol: &mut Protocol) -> Result<()> {
    let command = protocol
        .lingoes
        .iter_mut()
        .find(|l| l.id == 0)
        .context("RF certification patch: General lingo missing")?
        .commands
        .iter_mut()
        .find(|c| c.id == 0x39)
        .context("RF certification patch: SetFIDTokenValues missing")?;
    let tokens = command
        .nodes
        .iter_mut()
        .find_map(|node| match node {
            Node::Field(Field {
                key,
                kind: FieldKind::Tokens(tokens),
                ..
            }) if key == "tokens" => Some(tokens),
            _ => None,
        })
        .context("RF certification patch: tokens field missing")?;
    let token = tokens
        .iter_mut()
        .find(|t| t.fid_type == 0 && t.fid_subtype == 2)
        .context("RF certification patch: AccInfoToken missing")?;
    ensure!(
        token.nodes.iter().any(|node| matches!(node,
            Node::Field(Field { key, kind: FieldKind::Enum(values), .. })
            if key == "accInfoType" && values.items.iter().any(|v| v.value == 12)
        )),
        "RF certification patch: expected accInfoType 0x0C missing"
    );
    let expression = predicate::parse("accInfoType == 0x0C")?;
    // Do not silently duplicate/override this branch if a newer schema supplies it.
    if token.nodes.iter().any(|node| {
        matches!(node,
            Node::Predicate { expression: existing, .. } if *existing == expression
        )
    }) {
        return Ok(());
    }
    // iPod Accessory Protocol Interface Specification R46 (2012-09-12),
    // Accessory Info Type 0x0C and Table 3-74 (page 166): 32-bit mask.
    // Bit 2 (Class 3) and bits 7..31 are reserved; preserve their raw values.
    let bits = [
        (0, "Class1", "Class 1: iPhone, iPhone 3G, iPhone 3GS"),
        (1, "Class2", "Class 2: iPhone 4 (GSM model)"),
        (3, "Class4", "Class 4: iPhone 4 (CDMA model)"),
        (4, "Class5", "Class 5: iPhone 4S"),
        (5, "Class6", "Class 6: iPhone 5 (A1428 model)"),
        (6, "Class7", "Class 7: iPhone 5 (A1429 model)"),
    ]
    .into_iter()
    .map(|(index, name, description)| Bit {
        mask: 1 << index,
        name: name.into(),
        description: description.into(),
        required: false,
        deprecated: false,
    })
    .collect();
    token.nodes.push(Node::Predicate {
        expression,
        nodes: vec![Node::Field(Field {
            key: "rfCertificationDeclaration".into(),
            description: "Accessory RF certification declaration (R46 Table 3-74)".into(),
            length: None,
            count: None,
            kind: FieldKind::Bitfield(Bitfield {
                storage_ty: StorageType::U32,
                bits,
                groups: vec![],
            }),
        })],
    });
    Ok(())
}

// Each branch describes the option mask of the lingo selected by lingoID.
fn patch_lingo_option_descriptions(protocol: &mut Protocol) -> Result<()> {
    use super::ir::predicate::{PredicateExpr as E, PredicateToken as T};
    use std::collections::{BTreeMap, BTreeSet};
    let descriptions: BTreeMap<_, _> = protocol
        .lingoes
        .iter()
        .map(|lingo| (u64::from(lingo.id), format!("{} Options", lingo.name)))
        .collect();
    let command = protocol
        .lingoes
        .iter_mut()
        .find(|l| l.id == 0)
        .context("lingo options patch: General missing")?
        .commands
        .iter_mut()
        .find(|c| c.id == 0x4c)
        .context("lingo options patch: RetiPodOptionsForLingo missing")?;
    ensure!(
        command.nodes.len() == 18 && matches!(command.nodes[0], Node::Template),
        "lingo options patch: expected template, selector and 16 branches"
    );
    ensure!(
        matches!(&command.nodes[1], Node::Field(Field { key, kind: FieldKind::Scalar(StorageType::LingoId), .. }) if key == "lingoID"),
        "lingo options patch: expected lingoID selector"
    );
    let mut seen = BTreeSet::new();
    for node in &mut command.nodes[2..] {
        let Node::Predicate { expression, nodes } = node else {
            anyhow::bail!("lingo options patch: expected conditional branch")
        };
        let (id, description) = match expression {
            E::Eq(a, b) if matches!(&**a, E::Field(T::Field(key)) if key == "lingoID") => {
                let E::Integer(id) = &**b else {
                    anyhow::bail!("lingo options patch: expected literal lingoID")
                };
                ensure!(*id <= 14, "lingo options patch: unexpected lingoID {id}");
                (
                    *id,
                    descriptions
                        .get(id)
                        .with_context(|| format!("lingo options patch: lingo {id:#x} missing"))?
                        .clone(),
                )
            }
            E::Gt(a, b) if matches!(&**a, E::Field(T::Field(key)) if key == "lingoID") && **b == E::Integer(14) => {
                (15, "Unknown Lingo Options".into())
            }
            _ => anyhow::bail!("lingo options patch: unexpected selector {expression:?}"),
        };
        ensure!(seen.insert(id), "lingo options patch: duplicate branch {id}");
        ensure!(nodes.len() == 1, "lingo options patch: expected one optionBits field");
        let Node::Field(field) = &mut nodes[0] else {
            anyhow::bail!("lingo options patch: expected optionBits field")
        };
        ensure!(
            field.key == "optionBits" && matches!(&field.kind, FieldKind::Bitfield(bits) if bits.storage_ty == StorageType::U64),
            "lingo options patch: expected U64 optionBits mask"
        );
        ensure!(
            field.description == "Option Bits" || field.description == description,
            "lingo options patch: unexpected description {:?}",
            field.description
        );
        field.description = description;
    }
    ensure!(seen.len() == 16, "lingo options patch: incomplete branches");
    Ok(())
}

// R46 Table 3-57, pages 152–155. The class determines the meaning of
// preferenceSettingID in both commands and the iPod preference FID token.
fn patch_preference_descriptions(protocol: &mut Protocol) -> Result<()> {
    use super::ir::predicate::{PredicateExpr as E, PredicateToken as T};
    fn visit(nodes: &mut [Node]) -> Result<usize> {
        let mut patched = 0;
        for node in nodes {
            match node {
                Node::Predicate { expression, nodes } => {
                    if let E::Eq(a, b) = expression {
                        if matches!(&**a, E::Field(T::Field(key)) if key == "preferenceClassID") {
                            let E::Integer(id) = &**b else {
                                anyhow::bail!("preference descriptions patch: expected literal class ID")
                            };
                            let description = match *id {
                                0x00 => "Video Out Setting",
                                0x01 => "Screen Configuration",
                                0x02 => "Video Signal Format",
                                0x03 => "Line Out Usage",
                                0x08 => "Video Out Connection",
                                0x09 => "Closed Captioning",
                                0x0a => "Video Monitor Aspect Ratio",
                                0x0c => "Subtitles",
                                0x0d => "Video Alternate Audio Channel",
                                0x0f => "Pause On Power Removal",
                                0x14 => "VoiceOver",
                                0x16 => "AssistiveTouch",
                                _ => anyhow::bail!("preference descriptions patch: unexpected class {id:#x}"),
                            };
                            let mut count = 0;
                            for node in nodes.iter_mut() {
                                if let Node::Field(field) = node {
                                    if field.key == "preferenceSettingID" {
                                        ensure!(
                                            field.description == "Preference Setting" || field.description == description,
                                            "preference descriptions patch: unexpected description {:?}",
                                            field.description
                                        );
                                        field.description = description.into();
                                        count += 1;
                                    }
                                }
                            }
                            ensure!(count == 1, "preference descriptions patch: expected one setting for class {id:#x}");
                            patched += count;
                        }
                    }
                    patched += visit(nodes)?;
                }
                Node::Field(field) => match &mut field.kind {
                    FieldKind::Collection(nodes) | FieldKind::Records(nodes) => patched += visit(nodes)?,
                    FieldKind::Tokens(tokens) => {
                        for token in tokens {
                            patched += visit(&mut token.nodes)?;
                        }
                    }
                    _ => {}
                },
                Node::Template => {}
            }
        }
        Ok(patched)
    }
    let general = protocol
        .lingoes
        .iter_mut()
        .find(|l| l.id == 0)
        .context("preference descriptions patch: General missing")?;
    for id in [0x2a, 0x2b, 0x39] {
        let command = general
            .commands
            .iter_mut()
            .find(|c| c.id == id)
            .with_context(|| format!("preference descriptions patch: command {id:#x} missing"))?;
        ensure!(
            visit(&mut command.nodes)? == 12,
            "preference descriptions patch: expected twelve classes in command {id:#x}"
        );
    }
    Ok(())
}

// R46 Table 4-231 (page 344): userDataType 0x02 carries Gender,
// not the Preferred Unit System used by userDataType 0x00.
fn patch_user_data_gender_description(protocol: &mut Protocol) -> Result<()> {
    let command = protocol
        .lingoes
        .iter_mut()
        .find(|l| l.id == 9)
        .context("gender patch: Sports lingo missing")?
        .commands
        .iter_mut()
        .find(|c| c.id == 0x8a)
        .context("gender patch: SetUserData missing")?;
    let selector = predicate::parse("userDataType == 2")?;
    let nodes = command
        .nodes
        .iter_mut()
        .find_map(|node| match node {
            Node::Predicate { expression, nodes } if *expression == selector => Some(nodes),
            _ => None,
        })
        .context("gender patch: userDataType 2 branch missing")?;
    ensure!(
        nodes
            .iter()
            .filter(|n| !matches!(n, Node::Template))
            .count()
            == 1,
        "gender patch: expected one field"
    );
    let field = nodes
        .iter_mut()
        .find_map(|node| match node {
            Node::Field(field) => Some(field),
            _ => None,
        })
        .context("gender patch: expected enum field")?;
    ensure!(field.key == "preferredUnitSystem", "gender patch: unexpected key");
    let FieldKind::Enum(values) = &field.kind else {
        anyhow::bail!("gender patch: expected gender enum")
    };
    ensure!(
        values.storage_ty == StorageType::U8
            && values.items.len() == 3
            && values
                .items
                .iter()
                .map(|v| (v.value, v.description.as_str()))
                .eq([(0, "no information"), (1, "female"), (2, "male")]),
        "gender patch: unexpected gender values"
    );
    ensure!(
        field.description == "Preferred Unit System" || field.description == "Gender",
        "gender patch: unexpected description"
    );
    field.description = "Gender".into();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::{archive::KeyedArchive, generate_lingoes};
    use plist::Value;
    use std::io::Cursor;

    #[test]
    fn binary_schema_patch_is_idempotent_and_preserves_other_lingoes() -> Result<()> {
        let root = Value::from_reader(Cursor::new(include_bytes!("../Default.lktspec")))?;
        let mut protocol = Protocol::parse(&KeyedArchive::new(&root)?)?;
        let before = generate_lingoes(&protocol)?;
        apply(&mut protocol)?;
        let after = generate_lingoes(&protocol)?;
        assert_ne!(before[0], after[0]);
        for (old, new) in before[1..].iter().zip(&after[1..]) {
            if old.0 != "lingo_0x9.rs" && old.0 != "strings.rs" {
                assert_eq!(old, new);
            }
        }
        assert_ne!(before.last(), after.last());
        assert!(
            after
                .iter()
                .any(|(_, source)| source.contains("Gender(SetUserDataGender)"))
        );
        assert!(
            after[0]
                .1
                .contains("optional pub rf_certification_declaration: ")
        );
        assert!(after[0].1.contains("eq(&acc_info_type, &12u64)"));
        for lingo in &protocol.lingoes {
            assert!(after[0].1.contains(&format!("/// {} Options", lingo.name)));
        }
        assert!(after[0].1.contains("/// Unknown Lingo Options"));
        assert!(!after[0].1.contains("RetiPodOptionsForLingoOptionBits2"));
        assert!(!after[0].1.contains("PreferenceSetting2"));
        for owner in [
            "RetiPodPreferences",
            "SetiPodPreferences",
            "SetFIDTokenValuesTokensIPodPreferenceToken",
        ] {
            assert!(after[0].1.contains(&format!("{owner}ScreenConfiguration")));
        }
        apply(&mut protocol)?;
        assert_eq!(after, generate_lingoes(&protocol)?);
        Ok(())
    }
}
