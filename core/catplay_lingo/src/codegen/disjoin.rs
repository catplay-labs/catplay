//! Disjoins keep shared storage separate from original wire positions.
use super::ir::predicate_analysis::{Coverage, Overlap, check_coverage, check_overlap};
use super::*;
use std::collections::BTreeMap;
const LIMIT: usize = 100_000;

fn schema_project_wire(field: &Field, ty: &str, slots: &HashMap<String, u8>, disjoint: Option<&String>) -> Result<String> {
    let counted = field.count.as_ref().or(field.length.as_ref());
    let inner = if ty.starts_with("[u8; ") {
        "Iap1WireSpec::Scalar".to_owned()
    } else if let Some(count) = counted {
        if let Ok(count) = count.parse::<usize>() {
            format!("Iap1WireSpec::CountedLiteral({count})")
        } else {
            let slot = slots
                .get(count)
                .with_context(|| format!("wire length/count field {count:?} has no assigned slot"))?;
            format!("Iap1WireSpec::CountedField({slot})")
        }
    } else if ty == "String" || ty == "Vec<u8>" {
        "Iap1WireSpec::Raw".to_owned()
    } else if ty.starts_with("Vec<") {
        "Iap1WireSpec::CountedRemaining".to_owned()
    } else {
        "Iap1WireSpec::Scalar".to_owned()
    };
    Ok(disjoint.map_or(inner.clone(), |decode| {
        format!("Iap1WireSpec::Disjoint {{ variant: const {{ LingoString::require_raw(stringify!({decode})) }}, child: &{inner} }}")
    }))
}

fn flatten<'a>(nodes: &'a [Node], condition: PredicateExpr, out: &mut Vec<(&'a Field, PredicateExpr)>) {
    for node in nodes {
        match node {
            Node::Field(field) => out.push((field, condition.clone())),
            Node::Predicate { expression, nodes } => {
                let condition = if condition == PredicateExpr::Bool(true) {
                    expression.clone()
                } else {
                    PredicateExpr::And(Box::new(condition.clone()), Box::new(expression.clone()))
                };
                flatten(nodes, condition, out);
            }
            Node::Template => {}
        }
    }
}

fn same_type(a: &Field, b: &Field) -> bool {
    a.kind == b.kind && a.length == b.length && a.count == b.count
}

fn numeric_type(kind: &FieldKind) -> bool {
    match kind {
        FieldKind::Enum(_) | FieldKind::Bitfield(_) => true,
        FieldKind::Scalar(ty) => matches!(
            ty,
            StorageType::U8
                | StorageType::U16
                | StorageType::U32
                | StorageType::U64
                | StorageType::I8
                | StorageType::I16
                | StorageType::I32
                | StorageType::I64
                | StorageType::Bool
                | StorageType::CommandId
                | StorageType::LongCommandId
                | StorageType::LingoId
        ),
        _ => false,
    }
}

pub(super) fn emit(nodes: &[Node], owner: &str, names: &mut Names, aux: &mut String) -> Result<String> {
    let mut fields = Vec::new();
    flatten(nodes, PredicateExpr::Bool(true), &mut fields);
    let mut groups = BTreeMap::<&str, Vec<usize>>::new();
    for (index, (field, _)) in fields.iter().enumerate() {
        groups.entry(&field.key).or_default().push(index);
    }
    let non_numeric: HashSet<_> = groups
        .iter()
        .filter(|(_, indices)| indices.len() > 1 && indices.iter().any(|&i| !numeric_type(&fields[i].0.kind)))
        .map(|(&key, _)| key)
        .collect();
    for (index, (field, predicate)) in fields.iter().enumerate() {
        predicate.visit_prefix(&mut |token| -> Result<()> {
            if let FlatPredicateToken::Field(key) = token {
                if non_numeric.contains(key) {
                    bail!(
                        "{owner}: predicate at virtual index #{} (field {:?}) refers to non-numeric disjoint {key:?}",
                        index + 1,
                        field.key
                    );
                }
            }
            Ok(())
        })?;
        for dependency in [&field.length, &field.count].into_iter().flatten() {
            if non_numeric.contains(dependency.as_str()) {
                bail!(
                    "{owner}: length/count at virtual index #{} (field {:?}) refers to non-numeric disjoint {dependency:?}",
                    index + 1,
                    field.key
                );
            }
        }
    }
    let mut canonical: Vec<_> = (0..fields.len()).collect();
    for indices in groups.values().filter(|indices| indices.len() > 1) {
        let first = indices[0];
        let base = fields[first].0;
        for (position, &left) in indices.iter().enumerate() {
            for &right in &indices[position + 1..] {
                let result = check_overlap(&fields[left].1, &fields[right].1, LIMIT);
                if !matches!(result, Overlap::Disjoint) {
                    bail!(
                        "{owner}: disjoin {:?}, #{} / #{} not proven disjoint: {result:?}",
                        base.key,
                        left + 1,
                        right + 1
                    );
                }
            }
        }
        for &index in indices {
            canonical[index] = first;
        }
    }

    let field_bits: HashMap<usize, usize> = canonical
        .iter()
        .enumerate()
        .filter(|(index, first)| *index == **first)
        .enumerate()
        .map(|(bit, (index, _))| (index, bit))
        .collect();
    let field_slots: HashMap<String, u8> = fields
        .iter()
        .enumerate()
        .filter(|(index, _)| canonical[*index] == *index)
        .map(|(index, (field, _))| (field.key.clone(), field_bits[&index] as u8))
        .collect();
    let mut predicate_fields = HashSet::new();
    for (_, predicate) in &fields {
        predicate.visit_prefix(&mut |token| -> Result<()> {
            if let FlatPredicateToken::Field(key) = token {
                predicate_fields.insert(key.to_owned());
            }
            Ok(())
        })?;
    }
    for (field, _) in &fields {
        for dependency in [&field.length, &field.count].into_iter().flatten() {
            if dependency.parse::<usize>().is_err() {
                predicate_fields.insert(dependency.clone());
            }
        }
    }
    let mut types = HashMap::new();
    let mut alternatives = HashMap::<usize, (String, String, String)>::new();
    let mut rust_names = HashMap::new();
    let mut modes = HashMap::new();
    let mut field_names = HashSet::new();
    for (index, (field, _)) in fields
        .iter()
        .enumerate()
        .filter(|(i, _)| canonical[*i] == *i)
    {
        let base = field_name(&field.key);
        if !field_names.insert(base.clone()) {
            bail!(
                "{owner}: duplicate Rust field name {base:?} for schema key {:?}; legacy suffixed fields are forbidden, use disjoint storage",
                field.key
            );
        }
        let name = base;
        names.record(&field.key);
        rust_names.insert(index, name);
        let indices = &groups[field.key.as_str()];
        // Once a wrapper is needed, preserve distinct semantic variants even
        // when some children happen to have identical wire types (empty masks).
        // Entirely same-type groups still share a single field without a wrapper.
        let heterogeneous = indices.iter().any(|&i| !same_type(field, fields[i].0));
        let mut representatives: Vec<usize> = Vec::new();
        let mut child_for = HashMap::new();
        for &i in indices {
            let representative = representatives
                .iter()
                .copied()
                .find(|&r| same_type(fields[r].0, fields[i].0) && (!heterogeneous || fields[r].0.description == fields[i].0.description))
                .unwrap_or(i);
            if representative == i {
                representatives.push(i);
            }
            child_for.insert(i, representative);
        }
        if representatives.len() == 1 {
            types.insert(index, emit_field_type(field, owner, names, aux)?);
        } else {
            let wrapper = names.unique(format!("{owner}{}Disjoint", type_name(&field.key)));
            let mut variants = HashSet::new();
            let mut definitions = String::new();
            let mut child_definitions = HashMap::new();
            for &r in &representatives {
                let child = emit_field_type_named(fields[r].0, format!("{owner}{}", type_name(&fields[r].0.description)), names, aux)?;
                let variant = unique_name(type_name(&fields[r].0.description), &mut variants);
                names.record(&variant);
                let decode = format!("decode_{}", field_name(&variant));
                names.record(&decode);
                let encode = format!("encode_{}", field_name(&variant));
                writeln!(definitions, "        #[iap1_disjoint(decode = {decode}, encode = {encode})]")?;
                doc(&mut definitions, "        ", &fields[r].0.description)?;
                writeln!(definitions, "        {variant}({child}),")?;
                child_definitions.insert(r, (child, decode, encode));
            }
            let numeric = representatives
                .iter()
                .all(|&r| numeric_type(&fields[r].0.kind));
            writeln!(
                aux,
                "iap1_disjoint! {{\n    strings = LingoString;\n    #[iap1_disjoint(predicate = {})]\n    pub enum {wrapper} {{\n{definitions}    }}\n}}\n",
                if numeric { "integer" } else { "opaque" }
            )?;
            for &i in indices {
                alternatives.insert(i, child_definitions[&child_for[&i]].clone());
            }
            types.insert(index, wrapper);
        }
        let predicates: Vec<_> = fields
            .iter()
            .enumerate()
            .filter(|(i, _)| canonical[*i] == index)
            .map(|(_, (_, p))| p.clone())
            .collect();
        let mut length_selected = false;
        for predicate in &predicates {
            predicate.visit_prefix(&mut |token| -> Result<()> {
                length_selected |= matches!(token, FlatPredicateToken::AdjustedPayloadLength);
                Ok(())
            })?;
        }
        let mode = if length_selected {
            "length_optional"
        } else if matches!(check_coverage(&predicates, LIMIT), Coverage::Exhaustive) {
            "required"
        } else {
            "optional"
        };
        modes.insert(index, mode);
    }
    let mut expressions = HashMap::<usize, Vec<String>>::new();
    let mut references = HashMap::new();
    let mut available = HashMap::<String, Vec<PredicateExpr>>::new();
    let mut steps = String::new();
    for (index, (field, predicate)) in fields.iter().enumerate() {
        let first = canonical[index];
        validate_references(predicate, &references, owner, index + 1)?;
        // A merged value must have been read on every path reaching this predicate.
        predicate.visit_prefix(&mut |token| -> Result<()> {
            if let FlatPredicateToken::Field(key) = token {
                if groups
                    .get(key)
                    .is_some_and(|indices| indices.iter().any(|&i| canonical[i] != i))
                {
                    let prior = available
                        .get(key)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .reduce(|a, b| PredicateExpr::Or(Box::new(a), Box::new(b)))
                        .unwrap_or(PredicateExpr::Bool(false));
                    let result = check_overlap(predicate, &PredicateExpr::Not(Box::new(prior)), LIMIT);
                    if !matches!(result, Overlap::Disjoint) {
                        bail!(
                            "{owner}: predicate at virtual index #{} can refer to unread disjoin {key:?}: {result:?}",
                            index + 1
                        );
                    }
                }
            }
            Ok(())
        })?;
        for dependency in [&field.length, &field.count].into_iter().flatten() {
            if dependency.parse::<usize>().is_err() && !references.contains_key(dependency) {
                bail!("{owner}: wire at virtual index #{} refers forward to {dependency:?}", index + 1);
            }
        }
        let expression = predicate_expr_rust(predicate, &references);
        let program = super::predicate_program_rust(predicate, &field_slots)?;
        expressions
            .entry(first)
            .or_default()
            .push(expression.clone());
        let ty = &types[&first];
        let (wire, project_wire) = if let Some((child, decode, encode)) = alternatives.get(&index) {
            let wire = schema_wire(field, child, &references);
            let project_wire = schema_project_wire(field, child, &field_slots, Some(decode))?;
            (format!("disjoint({decode}, {encode}, {child}, [{wire}])"), project_wire)
        } else {
            (
                schema_wire(field, ty, &references),
                schema_project_wire(field, ty, &field_slots, None)?,
            )
        };
        let rust_name = &rust_names[&first];
        writeln!(
            steps,
            "            {} => {} {rust_name}: {ty} => {{ bit: {}, key: {:?}, wire: [{wire}], project_wire: {project_wire}, predicate_value: {}, predicate: {{ expr: {expression}, program: {program} }} }},",
            index + 1,
            modes[&first],
            field_bits[&first],
            field.key,
            predicate_fields.contains(&field.key)
        )?;
        references.insert(field.key.clone(), rust_name.clone());
        available
            .entry(field.key.clone())
            .or_default()
            .push(predicate.clone());
    }
    let mut out = String::from("        fields {\n");
    for (index, (field, _)) in fields
        .iter()
        .enumerate()
        .filter(|(i, _)| canonical[*i] == *i)
    {
        doc(&mut out, "            ", &field.description)?;
        let indices: Vec<_> = canonical
            .iter()
            .enumerate()
            .filter(|(_, first)| **first == index)
            .map(|(i, _)| (i + 1).to_string())
            .collect();
        writeln!(out, "            // Virtual schema indices: {}", indices.join(", "))?;
        let active = expressions[&index]
            .iter()
            .map(|e| format!("({e})"))
            .collect::<Vec<_>>()
            .join(" || ");
        writeln!(
            out,
            "            {} pub {}: {} => {{ bit: {}, key: {:?}, when: {active}, predicate_value: {} }},",
            modes[&index],
            rust_names[&index],
            types[&index],
            field_bits[&index],
            field.key,
            predicate_fields.contains(&field.key)
        )?;
    }
    writeln!(out, "        }}\n        steps {{\n{steps}        }}")?;
    Ok(out)
}

pub(super) fn validate_references(
    predicate: &PredicateExpr,
    references: &HashMap<String, String>,
    owner: &str,
    index: usize,
) -> Result<()> {
    predicate.visit_prefix(&mut |token| -> Result<()> {
        if let FlatPredicateToken::Field(key) = token {
            if !references.contains_key(key) {
                bail!("{owner}: predicate at virtual index #{index} refers forward or to missing field {key:?}");
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field(key: &str) -> Node {
        Node::Field(Field {
            key: key.into(),
            description: key.into(),
            length: None,
            count: None,
            kind: FieldKind::Scalar(StorageType::U8),
        })
    }
    fn equals(key: &str, value: u64) -> PredicateExpr {
        PredicateExpr::Eq(
            Box::new(PredicateExpr::Field(PredicateToken::Field(key.into()))),
            Box::new(PredicateExpr::Integer(value)),
        )
    }
    fn branch(predicate: PredicateExpr, key: &str) -> Node {
        Node::Predicate {
            expression: predicate,
            nodes: vec![field(key)],
        }
    }
    fn generate(nodes: &[Node]) -> Result<String> {
        emit_fields(nodes, "Test", &mut Names::default(), &mut String::new())
    }

    #[test]
    fn opaque_disjoint_is_valid_until_used_as_a_predicate_operand() {
        let bytes = |length: &str| {
            Node::Field(Field {
                key: "payload".into(),
                description: "payload".into(),
                length: Some(length.into()),
                count: None,
                kind: FieldKind::Scalar(StorageType::Data),
            })
        };
        let mut nodes = vec![
            field("selector"),
            Node::Predicate {
                expression: equals("selector", 0),
                nodes: vec![bytes("16")],
            },
            Node::Predicate {
                expression: PredicateExpr::Not(Box::new(equals("selector", 0))),
                nodes: vec![bytes("20")],
            },
        ];
        assert!(generate(&nodes).is_ok());
        nodes.push(branch(equals("payload", 1), "dependent"));
        let error = generate(&nodes).unwrap_err().to_string();
        assert!(error.contains("non-numeric disjoint \"payload\""), "{error}");
        assert!(error.contains("virtual index #4"), "{error}");
        nodes.pop();
        nodes.push(Node::Field(Field {
            key: "dependent".into(),
            description: "dependent".into(),
            length: Some("payload".into()),
            count: None,
            kind: FieldKind::Scalar(StorageType::Data),
        }));
        let error = generate(&nodes).unwrap_err().to_string();
        assert!(error.contains("length/count"), "{error}");
    }

    #[test]
    fn reject_rust_name_collisions_with_and_without_duplicate_keys() {
        let mut nodes = vec![field("fooBar"), field("foo_bar")];
        assert!(
            generate(&nodes)
                .unwrap_err()
                .to_string()
                .contains("legacy suffixed fields are forbidden")
        );
        nodes.insert(0, field("selector"));
        nodes.push(branch(equals("selector", 0), "value"));
        nodes.push(branch(equals("selector", 1), "value"));
        assert!(
            generate(&nodes)
                .unwrap_err()
                .to_string()
                .contains("legacy suffixed fields are forbidden")
        );
    }

    #[test]
    fn coverage_controls_shared_storage_and_indices_are_preserved() {
        let condition = equals("selector", 0);
        let nodes = vec![
            field("selector"),
            branch(condition.clone(), "value"),
            field("marker"),
            branch(PredicateExpr::Not(Box::new(condition)), "value"),
        ];
        let source = generate(&nodes).unwrap();
        assert!(source.contains("required pub value: u8"));
        assert!(source.contains("2 => required value:"));
        assert!(source.contains("4 => required value:"));
        assert!(source.contains("2 => required value: u8 => { bit: 1,"));
        assert!(source.contains("4 => required value: u8 => { bit: 1,"));
        assert!(source.contains("3 => required marker: u8 => { bit: 2,"));
        let nodes = vec![
            field("selector"),
            branch(equals("selector", 0), "value"),
            field("marker"),
            branch(equals("selector", 1), "value"),
        ];
        assert!(generate(&nodes).unwrap().contains("optional pub value: u8"));
    }

    #[test]
    fn reject_forward_reference_even_when_storage_is_declared_up_front() {
        let nodes = vec![
            field("selector"),
            branch(equals("later", 0), "value"),
            field("later"),
            branch(equals("later", 1), "value"),
        ];
        assert!(
            generate(&nodes)
                .unwrap_err()
                .to_string()
                .contains("refers forward")
        );
    }

    #[test]
    fn reject_reference_to_merged_field_before_its_active_position() {
        let nodes = vec![
            field("selector"),
            branch(equals("selector", 0), "value"),
            branch(equals("value", 1), "dependent"),
            branch(equals("selector", 1), "value"),
        ];
        assert!(
            generate(&nodes)
                .unwrap_err()
                .to_string()
                .contains("unread disjoin")
        );
    }

    #[test]
    fn different_full_enum_types_use_untagged_disjoint() {
        use super::super::ir::EnumItem;
        let enum_field = |value| {
            Node::Field(Field {
                key: "value".into(),
                description: "value".into(),
                length: None,
                count: None,
                kind: FieldKind::Enum(EnumField {
                    storage_ty: StorageType::U8,
                    items: vec![EnumItem {
                        value,
                        name: "Value".into(),
                        description: "Value".into(),
                        deprecated: false,
                    }],
                    ranges: vec![],
                }),
            })
        };
        let nodes = vec![
            field("selector"),
            Node::Predicate {
                expression: equals("selector", 0),
                nodes: vec![enum_field(0)],
            },
            Node::Predicate {
                expression: equals("selector", 1),
                nodes: vec![enum_field(1)],
            },
        ];
        let source = generate(&nodes).unwrap();
        assert!(source.contains("steps {"));
        assert!(source.contains("optional pub value: TestValueDisjoint"));
        assert!(source.contains("disjoint("));
        assert!(!source.contains("pub value_2:"));
    }
}
