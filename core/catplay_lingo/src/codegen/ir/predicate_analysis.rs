//! Exact finite partition for field/literal comparisons; conservative elsewhere.
//!
//! Analysis uses the generator's predicate AST and prefix tokens. Integer domains
//! are unsigned u64, not the narrower or signed storage types of schema fields.
//! Future field deduplication must require proven disjointness before sharing
//! storage, and check coverage separately before removing fallback storage.
use super::predicate::{FlatPredicateToken as T, PredicateExpr as E, PredicateToken as F};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Integer(u64),
    Bool(bool),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlap {
    Disjoint,
    Overlapping { witness: BTreeMap<String, Value> },
    Unknown { reason: String },
}
impl Overlap {
    /// Only a proof of disjointness permits sharing storage.
    pub fn can_share_storage(&self) -> bool {
        matches!(self, Self::Disjoint)
    }
}
/// Coverage is independent of pairwise disjointness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coverage {
    Exhaustive,
    Gap { witness: BTreeMap<String, Value> },
    Unknown { reason: String },
}
impl Coverage {
    /// Unknown conservatively retains fallback storage.
    pub fn needs_fallback(&self) -> bool {
        !matches!(self, Self::Exhaustive)
    }
}
/// Checks whether any assignment makes all predicates false.
/// Empty input has a gap. The domain is the same as check_overlap (u64/bool).
pub fn check_coverage(predicates: &[E], max_assignments: usize) -> Coverage {
    let union = predicates
        .iter()
        .cloned()
        .reduce(|a, b| E::Or(Box::new(a), Box::new(b)))
        .unwrap_or(E::Bool(false));
    match check_overlap(&E::Not(Box::new(union)), &E::Bool(true), max_assignments) {
        Overlap::Disjoint => Coverage::Exhaustive,
        Overlap::Overlapping { witness } => Coverage::Gap { witness },
        Overlap::Unknown { reason } => Coverage::Unknown { reason },
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Integer,
    Bool,
}
fn key(f: &F) -> String {
    // Namespace prevents synthetic tokens aliasing ordinary Field names.
    match f {
        F::Field(n) => format!("field:{n}"),
        F::AdjustedPayloadLength => "synthetic:adjustedPayloadLength".into(),
        F::HasTransactionId => "synthetic:hasTransactionID".into(),
    }
}
fn field_kind(f: &F, kind: Kind, fields: &mut BTreeMap<String, Kind>) -> Result<(), String> {
    if matches!(f, F::AdjustedPayloadLength) && kind != Kind::Integer || matches!(f, F::HasTransactionId) && kind != Kind::Bool {
        return Err("incorrect synthetic field type".into());
    }
    if fields.insert(key(f), kind).is_some_and(|old| old != kind) {
        return Err("field used as both integer and boolean".into());
    }
    Ok(())
}
fn literal(e: &E) -> Option<Value> {
    match e {
        E::Integer(v) => Some(Value::Integer(*v)),
        E::Bool(v) => Some(Value::Bool(*v)),
        _ => None,
    }
}
fn inspect(e: &E, fields: &mut BTreeMap<String, Kind>, constants: &mut BTreeSet<u64>) -> Result<(), String> {
    match e {
        E::Bool(_) => Ok(()),
        E::Field(f) => field_kind(f, Kind::Bool, fields),
        E::Not(a) => inspect(a, fields, constants),
        E::And(a, b) | E::Or(a, b) => {
            inspect(a, fields, constants)?;
            inspect(b, fields, constants)
        }
        E::Eq(a, b) | E::Ne(a, b) | E::Lt(a, b) | E::Le(a, b) | E::Gt(a, b) | E::Ge(a, b) => {
            let equality = matches!(e, E::Eq(..) | E::Ne(..));
            let (f, v) = match (&**a, &**b) {
                (E::Field(f), v) | (v, E::Field(f)) if literal(v).is_some() => (Some(f), literal(v).unwrap()),
                (a, b) if literal(a).is_some() && literal(b).is_some() => {
                    let a = literal(a).unwrap();
                    let b = literal(b).unwrap();
                    if std::mem::discriminant(&a) != std::mem::discriminant(&b) || !equality && matches!(a, Value::Bool(_)) {
                        return Err("invalid literal comparison".into());
                    }
                    return Ok(());
                }
                _ => {
                    return Err("analysis supports comparisons of a field with a literal, or two literals".into());
                }
            };
            let kind = match v {
                Value::Integer(n) => {
                    constants.insert(n);
                    Kind::Integer
                }
                Value::Bool(_) if equality => Kind::Bool,
                _ => return Err("boolean ordering unsupported".into()),
            };
            field_kind(f.unwrap(), kind, fields)
        }
        _ => Err("integer used as a predicate".into()),
    }
}
fn value(e: &E, env: &BTreeMap<String, Value>) -> Value {
    match e {
        E::Integer(n) => Value::Integer(*n),
        E::Bool(v) => Value::Bool(*v),
        E::Field(f) => env[&key(f)],
        E::Not(a) => Value::Bool(!truth(a, env)),
        E::And(a, b) => Value::Bool(truth(a, env) && truth(b, env)),
        E::Or(a, b) => Value::Bool(truth(a, env) || truth(b, env)),
        E::Eq(a, b) => Value::Bool(value(a, env) == value(b, env)),
        E::Ne(a, b) => Value::Bool(value(a, env) != value(b, env)),
        E::Lt(a, b) | E::Le(a, b) | E::Gt(a, b) | E::Ge(a, b) => {
            let (Value::Integer(a), Value::Integer(b)) = (value(a, env), value(b, env)) else {
                unreachable!()
            };
            Value::Bool(match e {
                E::Lt(..) => a < b,
                E::Le(..) => a <= b,
                E::Gt(..) => a > b,
                _ => a >= b,
            })
        }
    }
}
fn truth(e: &E, env: &BTreeMap<String, Value>) -> bool {
    let Value::Bool(v) = value(e, env) else { unreachable!() };
    v
}
/// Numeric fields are unsigned u64; no integer/boolean coercion.
/// Limit bounds Cartesian enumeration, never changes a failure into a proof.
pub fn check_overlap(left: &E, right: &E, max_assignments: usize) -> Overlap {
    let mut fields = BTreeMap::new();
    let mut constants = BTreeSet::new();
    if let Err(reason) = inspect(left, &mut fields, &mut constants).and_then(|()| inspect(right, &mut fields, &mut constants)) {
        return Overlap::Unknown { reason };
    }
    let mut integers = BTreeSet::from([0, u64::MAX]);
    for n in constants {
        integers.insert(n);
        if let Some(v) = n.checked_sub(1) {
            integers.insert(v);
        }
        if let Some(v) = n.checked_add(1) {
            integers.insert(v);
        }
    }
    // Every literal comparison has constant truth on each interval between
    // constants. Boundary values and adjacent values represent every interval.
    let domains: Vec<_> = fields
        .into_iter()
        .map(|(name, kind)| {
            (
                name,
                match kind {
                    Kind::Bool => vec![Value::Bool(false), Value::Bool(true)],
                    Kind::Integer => integers.iter().copied().map(Value::Integer).collect(),
                },
            )
        })
        .collect();
    let mut indices = vec![0; domains.len()];
    let mut tried = 0;
    loop {
        if tried >= max_assignments {
            return Overlap::Unknown {
                reason: format!("assignment limit reached ({max_assignments})"),
            };
        }
        tried += 1;
        let env: BTreeMap<_, _> = domains
            .iter()
            .zip(&indices)
            .map(|((name, values), i)| (name.clone(), values[*i]))
            .collect();
        if truth(left, &env) && truth(right, &env) {
            return Overlap::Overlapping { witness: env };
        }
        let mut pos = 0;
        while pos < indices.len() {
            indices[pos] += 1;
            if indices[pos] < domains[pos].1.len() {
                break;
            }
            indices[pos] = 0;
            pos += 1;
        }
        if pos == indices.len() {
            return Overlap::Disjoint;
        }
    }
}
/// Rebuild the existing fixed-arity prefix format, rejecting malformed streams.
pub fn from_prefix(tokens: &[T<'_>]) -> Result<E, String> {
    fn read(tokens: &[T<'_>], offset: &mut usize, depth: usize) -> Result<E, String> {
        if depth > 256 {
            return Err("prefix nesting limit exceeded".into());
        }
        let t = *tokens.get(*offset).ok_or("missing prefix operand")?;
        *offset += 1;
        Ok(match t {
            T::Field(n) => E::Field(F::Field(n.into())),
            T::AdjustedPayloadLength => E::Field(F::AdjustedPayloadLength),
            T::HasTransactionId => E::Field(F::HasTransactionId),
            T::Integer(n) => E::Integer(u64::from(n)),
            T::Bool(v) => E::Bool(v),
            T::Not => E::Not(Box::new(read(tokens, offset, depth + 1)?)),
            op => {
                let a = Box::new(read(tokens, offset, depth + 1)?);
                let b = Box::new(read(tokens, offset, depth + 1)?);
                match op {
                    T::Eq => E::Eq(a, b),
                    T::Ne => E::Ne(a, b),
                    T::Lt => E::Lt(a, b),
                    T::Le => E::Le(a, b),
                    T::Gt => E::Gt(a, b),
                    T::Ge => E::Ge(a, b),
                    T::And => E::And(a, b),
                    T::Or => E::Or(a, b),
                    _ => unreachable!(),
                }
            }
        })
    }
    let mut offset = 0;
    let e = read(tokens, &mut offset, 0)?;
    if offset != tokens.len() {
        return Err("trailing prefix tokens".into());
    }
    Ok(e)
}
#[cfg(test)]
mod tests {
    use super::super::predicate::parse;
    use super::*;
    fn check(a: &str, b: &str) -> Overlap {
        check_overlap(&parse(a).unwrap(), &parse(b).unwrap(), 100_000)
    }
    #[test]
    fn disjoint_cases() {
        for (a, b) in [
            ("lingoID == 3", "lingoID == 4"),
            ("lingoID == 3 OR lingoID == 4", "lingoID == 5"),
            ("x < 10", "x >= 10"),
            ("NOT (x == 3)", "x == 3"),
            ("x <= 1 AND x != 0 AND x != 1", "TRUE"),
            ("x < 0", "TRUE"),
            ("x > 18446744073709551615", "TRUE"),
            ("hasTransactionID", "hasTransactionID == NO"),
        ] {
            assert_eq!(check(a, b), Overlap::Disjoint, "{a}, {b}");
        }
    }
    #[test]
    fn witnesses() {
        for (a, b) in [
            ("x >= 3", "x <= 3"),
            ("x > 3", "x < 5"),
            ("x == 3 OR x == 4", "x == 4"),
            ("x == 3", "y == 4"),
            ("NOT (x < 3 OR x > 5)", "x != 3"),
            ("hasTransactionID == YES", "TRUE"),
        ] {
            let Overlap::Overlapping { witness } = check(a, b) else {
                panic!("{a}, {b}")
            };
            assert!(truth(&parse(a).unwrap(), &witness) && truth(&parse(b).unwrap(), &witness));
        }
    }
    #[test]
    fn conservative_unknown() {
        for (a, b) in [
            ("x == y", "TRUE"),
            ("x == 3", "x == YES"),
            ("hasTransactionID == 3", "TRUE"),
            ("(x == 3) == YES", "TRUE"),
        ] {
            assert!(matches!(check(a, b), Overlap::Unknown { .. }));
        }
        assert!(matches!(
            check_overlap(&parse("x == 3").unwrap(), &parse("x == 4").unwrap(), 1),
            Overlap::Unknown { .. }
        ));
    }
    #[test]
    fn prefix_and_errors() {
        let a = from_prefix(&[T::Eq, T::Field("lingoID"), T::Integer(3)]).unwrap();
        let b = from_prefix(&[T::Eq, T::Field("lingoID"), T::Integer(4)]).unwrap();
        assert!(check_overlap(&a, &b, 100).can_share_storage());
        for tokens in [vec![], vec![T::Eq, T::Integer(3)], vec![T::Bool(true), T::Bool(false)]] {
            assert!(from_prefix(&tokens).is_err());
        }
    }
    #[test]
    fn existing_prefix_visitor_round_trips() {
        let expression = parse("adjustedPayloadLength >= 17 AND NOT hasTransactionID OR x != 3").unwrap();
        let mut tokens = Vec::new();
        expression
            .visit_prefix(&mut |token| {
                tokens.push(token);
                Ok::<_, ()>(())
            })
            .unwrap();
        assert_eq!(from_prefix(&tokens).unwrap(), expression);
    }

    #[test]
    fn synthetic_tokens_do_not_alias_named_fields() {
        let left = E::Field(F::HasTransactionId);
        let right = E::Not(Box::new(E::Field(F::Field("hasTransactionID".into()))));
        let Overlap::Overlapping { witness } = check_overlap(&left, &right, 4) else {
            panic!("synthetic and ordinary fields must be independent")
        };
        assert_eq!(witness["synthetic:hasTransactionID"], Value::Bool(true));
        assert_eq!(witness["field:hasTransactionID"], Value::Bool(false));
    }

    #[test]
    fn assignment_limit_and_prefix_depth_are_conservative() {
        assert!(matches!(check_overlap(&E::Bool(false), &E::Bool(true), 0), Overlap::Unknown { .. }));
        assert_eq!(check_overlap(&E::Bool(false), &E::Bool(true), 1), Overlap::Disjoint);
        let mut tokens = vec![T::Not; 257];
        tokens.push(T::Bool(true));
        assert!(from_prefix(&tokens).is_err());
    }

    #[test]
    fn partition_matches_small_exhaustive_domain() {
        let expressions = [
            "x == 0",
            "x != 1",
            "x < 2",
            "x <= 3",
            "x > 2",
            "x >= 4",
            "NOT (x < 2)",
            "x == 1 OR x == 3",
            "x > 0 AND x < 3",
        ];
        for a in expressions {
            for b in expressions {
                let a = parse(a).unwrap();
                let b = parse(b).unwrap();
                let expected = (0..=5).any(|x| {
                    let env = BTreeMap::from([("field:x".into(), Value::Integer(x))]);
                    truth(&a, &env) && truth(&b, &env)
                });
                assert_eq!(matches!(check_overlap(&a, &b, 100), Overlap::Overlapping { .. }), expected);
            }
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::super::predicate::parse;
    use super::*;
    fn coverage(sources: &[&str]) -> Coverage {
        check_coverage(
            &sources
                .iter()
                .map(|s| parse(s).unwrap())
                .collect::<Vec<_>>(),
            100_000,
        )
    }
    #[test]
    fn exhaustive_partitions() {
        for sources in [
            vec!["x < 10", "x >= 10"],
            vec!["hasTransactionID", "NOT hasTransactionID"],
            vec!["x == 3", "x != 3"],
            vec!["TRUE"],
            vec!["x <= 18446744073709551615"],
        ] {
            assert_eq!(coverage(&sources), Coverage::Exhaustive);
            assert!(!coverage(&sources).needs_fallback());
        }
    }
    #[test]
    fn gap_has_valid_witness() {
        for sources in [
            vec!["lingoID == 3", "lingoID == 4"],
            vec!["x < 10", "x > 10"],
            vec!["FALSE"],
            vec![],
        ] {
            let result = coverage(&sources);
            assert!(result.needs_fallback());
            let Coverage::Gap { witness } = result else {
                panic!("expected gap")
            };
            assert!(sources.iter().all(|s| !truth(&parse(s).unwrap(), &witness)));
        }
        assert_eq!(
            coverage(&["x < 10", "x > 10"]),
            Coverage::Gap {
                witness: BTreeMap::from([("field:x".into(), Value::Integer(10))])
            }
        );
    }
    #[test]
    fn unknown_keeps_fallback() {
        assert!(matches!(coverage(&["x == y"]), Coverage::Unknown { .. }));
        let result = check_coverage(&[parse("x < 10 OR x >= 10").unwrap()], 1);
        assert!(matches!(result, Coverage::Unknown { .. }));
        assert!(result.needs_fallback());
    }
    #[test]
    fn coverage_does_not_prove_disjointness() {
        assert_eq!(coverage(&["x <= 10", "x >= 10"]), Coverage::Exhaustive);
        assert!(matches!(
            check_overlap(&parse("x <= 10").unwrap(), &parse("x >= 10").unwrap(), 100_000),
            Overlap::Overlapping { .. }
        ));
    }
}
