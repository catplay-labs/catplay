//! Parser for the predicate subset used by LingoKit schemas. The generator
//! uses this AST and its flat prefix traversal internally for analysis.
//! Generated macros receive only the compiled Rust predicate expression.

use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PredicateToken {
    AdjustedPayloadLength,
    HasTransactionId,
    Field(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PredicateExpr {
    Field(PredicateToken),
    Integer(u64),
    Bool(bool),
    Eq(Box<Self>, Box<Self>),
    Ne(Box<Self>, Box<Self>),
    Lt(Box<Self>, Box<Self>),
    Le(Box<Self>, Box<Self>),
    Gt(Box<Self>, Box<Self>),
    Ge(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),
}

/// Prefix (Polish notation) atoms. Operators have fixed arity: `Not` takes
/// one operand, comparisons and boolean operators take two. A consumer can
/// rebuild an expression with a single cursor and no precedence rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatPredicateToken<'a> {
    Field(&'a str),
    AdjustedPayloadLength,
    HasTransactionId,
    Integer(u64),
    Bool(bool),
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
}

impl PredicateExpr {
    /// Visits a flat prefix stream without allocating a token vector.
    pub fn visit_prefix<'a, E>(
        &'a self,
        emit: &mut impl FnMut(FlatPredicateToken<'a>) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), E> {
        use FlatPredicateToken as T;
        match self {
            Self::Field(PredicateToken::Field(name)) => emit(T::Field(name)),
            Self::Field(PredicateToken::AdjustedPayloadLength) => emit(T::AdjustedPayloadLength),
            Self::Field(PredicateToken::HasTransactionId) => emit(T::HasTransactionId),
            Self::Integer(value) => emit(T::Integer(*value)),
            Self::Bool(value) => emit(T::Bool(*value)),
            Self::Not(inner) => {
                emit(T::Not)?;
                inner.visit_prefix(emit)
            }
            Self::Eq(a, b) => visit_binary(T::Eq, a, b, emit),
            Self::Ne(a, b) => visit_binary(T::Ne, a, b, emit),
            Self::Lt(a, b) => visit_binary(T::Lt, a, b, emit),
            Self::Le(a, b) => visit_binary(T::Le, a, b, emit),
            Self::Gt(a, b) => visit_binary(T::Gt, a, b, emit),
            Self::Ge(a, b) => visit_binary(T::Ge, a, b, emit),
            Self::And(a, b) => visit_binary(T::And, a, b, emit),
            Self::Or(a, b) => visit_binary(T::Or, a, b, emit),
        }
    }
}

fn visit_binary<'a, E>(
    op: FlatPredicateToken<'a>,
    left: &'a PredicateExpr,
    right: &'a PredicateExpr,
    emit: &mut impl FnMut(FlatPredicateToken<'a>) -> std::result::Result<(), E>,
) -> std::result::Result<(), E> {
    emit(op)?;
    left.visit_prefix(emit)?;
    right.visit_prefix(emit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Lexeme {
    Identifier(String),
    Integer(u64),
    Bool(bool),
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
}

pub fn parse(source: &str) -> Result<PredicateExpr> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        tokens: &tokens,
        offset: 0,
    };
    let expression = parser.or()?;
    if let Some(token) = parser.peek() {
        bail!("unexpected token {token:?} after predicate expression");
    }
    Ok(expression)
}

/// Ensure predicate constants can be represented by the compact runtime token.
/// Schema loading calls this before retaining a predicate expression.
pub fn validate_flat_integer_range(expression: &PredicateExpr) -> Result<()> {
    match expression {
        PredicateExpr::Integer(value) if *value > u64::from(u8::MAX) => {
            bail!("predicate integer {value} exceeds u8::MAX")
        }
        PredicateExpr::Eq(a, b)
        | PredicateExpr::Ne(a, b)
        | PredicateExpr::Lt(a, b)
        | PredicateExpr::Le(a, b)
        | PredicateExpr::Gt(a, b)
        | PredicateExpr::Ge(a, b)
        | PredicateExpr::And(a, b)
        | PredicateExpr::Or(a, b) => {
            validate_flat_integer_range(a)?;
            validate_flat_integer_range(b)?;
        }
        PredicateExpr::Not(inner) => validate_flat_integer_range(inner)?,
        PredicateExpr::Field(_) | PredicateExpr::Integer(_) | PredicateExpr::Bool(_) => {}
    }
    Ok(())
}

fn lex(source: &str) -> Result<Vec<Lexeme>> {
    let bytes = source.as_bytes();
    let mut pos = 0;
    let mut result = Vec::new();
    while pos < bytes.len() {
        if bytes[pos].is_ascii_whitespace() {
            pos += 1;
            continue;
        }
        let rest = &source[pos..];
        let pair = if rest.starts_with("==") {
            Some(Lexeme::Eq)
        } else if rest.starts_with("!=") {
            Some(Lexeme::Ne)
        } else if rest.starts_with("<=") {
            Some(Lexeme::Le)
        } else if rest.starts_with(">=") {
            Some(Lexeme::Ge)
        } else if rest.starts_with("&&") {
            Some(Lexeme::And)
        } else if rest.starts_with("||") {
            Some(Lexeme::Or)
        } else {
            None
        };
        if let Some(token) = pair {
            result.push(token);
            pos += 2;
            continue;
        }
        let single = match bytes[pos] {
            b'<' => Some(Lexeme::Lt),
            b'>' => Some(Lexeme::Gt),
            b'!' => Some(Lexeme::Not),
            b'(' => Some(Lexeme::LeftParen),
            b')' => Some(Lexeme::RightParen),
            _ => None,
        };
        if let Some(token) = single {
            result.push(token);
            pos += 1;
            continue;
        }
        if bytes[pos].is_ascii_digit() {
            let start = pos;
            if bytes[pos] == b'0' && bytes.get(pos + 1).is_some_and(|v| *v == b'x' || *v == b'X') {
                pos += 2;
                let digits = pos;
                while bytes.get(pos).is_some_and(u8::is_ascii_hexdigit) {
                    pos += 1;
                }
                if pos == digits {
                    bail!("expected hexadecimal digits at offset {start}");
                }
                let value = u64::from_str_radix(&source[digits..pos], 16)
                    .map_err(|_| anyhow::anyhow!("hexadecimal integer out of range at offset {start}"))?;
                result.push(Lexeme::Integer(value));
            } else {
                while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                    pos += 1;
                }
                let value = source[start..pos]
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("decimal integer out of range at offset {start}"))?;
                result.push(Lexeme::Integer(value));
            }
            continue;
        }
        if bytes[pos].is_ascii_alphabetic() || bytes[pos] == b'_' {
            let start = pos;
            pos += 1;
            while bytes
                .get(pos)
                .is_some_and(|v| v.is_ascii_alphanumeric() || *v == b'_')
            {
                pos += 1;
            }
            let word = &source[start..pos];
            result.push(match word.to_ascii_uppercase().as_str() {
                "AND" => Lexeme::And,
                "OR" => Lexeme::Or,
                "NOT" => Lexeme::Not,
                "YES" | "TRUE" => Lexeme::Bool(true),
                "NO" | "FALSE" => Lexeme::Bool(false),
                _ => Lexeme::Identifier(word.to_owned()),
            });
            continue;
        }
        bail!("unsupported character {:?} at offset {pos}", source[pos..].chars().next());
    }
    Ok(result)
}

struct Parser<'a> {
    tokens: &'a [Lexeme],
    offset: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Lexeme> {
        self.tokens.get(self.offset)
    }

    fn take(&mut self) -> Option<&Lexeme> {
        let token = self.tokens.get(self.offset);
        if token.is_some() {
            self.offset += 1;
        }
        token
    }

    fn or(&mut self) -> Result<PredicateExpr> {
        let mut left = self.and()?;
        while self.peek() == Some(&Lexeme::Or) {
            self.take();
            left = PredicateExpr::Or(Box::new(left), Box::new(self.and()?));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<PredicateExpr> {
        let mut left = self.comparison()?;
        while self.peek() == Some(&Lexeme::And) {
            self.take();
            left = PredicateExpr::And(Box::new(left), Box::new(self.comparison()?));
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<PredicateExpr> {
        let left = self.unary()?;
        let op = match self.peek() {
            Some(Lexeme::Eq | Lexeme::Ne | Lexeme::Lt | Lexeme::Le | Lexeme::Gt | Lexeme::Ge) => self.take().cloned(),
            _ => None,
        };
        let Some(op) = op else {
            return Ok(left);
        };
        let right = self.unary()?;
        let pair = (Box::new(left), Box::new(right));
        Ok(match op {
            Lexeme::Eq => PredicateExpr::Eq(pair.0, pair.1),
            Lexeme::Ne => PredicateExpr::Ne(pair.0, pair.1),
            Lexeme::Lt => PredicateExpr::Lt(pair.0, pair.1),
            Lexeme::Le => PredicateExpr::Le(pair.0, pair.1),
            Lexeme::Gt => PredicateExpr::Gt(pair.0, pair.1),
            Lexeme::Ge => PredicateExpr::Ge(pair.0, pair.1),
            _ => unreachable!(),
        })
    }

    fn unary(&mut self) -> Result<PredicateExpr> {
        if self.peek() == Some(&Lexeme::Not) {
            self.take();
            return Ok(PredicateExpr::Not(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<PredicateExpr> {
        match self.take().cloned() {
            Some(Lexeme::Identifier(name)) => Ok(PredicateExpr::Field(match name.as_str() {
                "adjustedPayloadLength" => PredicateToken::AdjustedPayloadLength,
                "hasTransactionID" => PredicateToken::HasTransactionId,
                _ => PredicateToken::Field(name),
            })),
            Some(Lexeme::Integer(value)) => Ok(PredicateExpr::Integer(value)),
            Some(Lexeme::Bool(value)) => Ok(PredicateExpr::Bool(value)),
            Some(Lexeme::LeftParen) => {
                let expression = self.or()?;
                if self.take() != Some(&Lexeme::RightParen) {
                    bail!("missing closing ')' in predicate");
                }
                Ok(expression)
            }
            token => bail!("expected predicate field, literal, or '('; got {token:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str) -> PredicateExpr {
        PredicateExpr::Field(PredicateToken::Field(name.to_owned()))
    }

    #[test]
    fn respects_precedence_and_parentheses() -> Result<()> {
        use PredicateExpr as E;
        assert_eq!(
            parse("!a == 0 || b < 0x10 && (c >= 2 || d != 3)")?,
            E::Or(
                Box::new(E::Eq(Box::new(E::Not(Box::new(field("a")))), Box::new(E::Integer(0)))),
                Box::new(E::And(
                    Box::new(E::Lt(Box::new(field("b")), Box::new(E::Integer(16)))),
                    Box::new(E::Or(
                        Box::new(E::Ge(Box::new(field("c")), Box::new(E::Integer(2)))),
                        Box::new(E::Ne(Box::new(field("d")), Box::new(E::Integer(3)))),
                    )),
                )),
            )
        );
        Ok(())
    }

    #[test]
    fn accepts_legacy_words_and_synthetic_fields() -> Result<()> {
        use PredicateExpr as E;
        assert_eq!(
            parse("hasTransactionID == YES AND adjustedPayloadLength <= 0x11")?,
            E::And(
                Box::new(E::Eq(Box::new(E::Field(PredicateToken::HasTransactionId)), Box::new(E::Bool(true)))),
                Box::new(E::Le(
                    Box::new(E::Field(PredicateToken::AdjustedPayloadLength)),
                    Box::new(E::Integer(17))
                )),
            )
        );
        assert!(matches!(parse("NO OR NOT FALSE")?, E::Or(_, _)));
        Ok(())
    }

    #[test]
    fn rejects_incomplete_or_unknown_input() {
        for source in ["", "a ==", "(a == 1", "a &&& b", "0x", "a @ 2", "a < b < c"] {
            assert!(parse(source).is_err(), "{source:?}");
        }
    }

    #[test]
    fn integer_predicate_literals_must_fit_u8() -> Result<()> {
        assert!(validate_flat_integer_range(&parse("field == 255")?).is_ok());
        assert!(validate_flat_integer_range(&parse("field == 0xff")?).is_ok());
        for source in ["field == 256", "field == 0x100"] {
            let error = validate_flat_integer_range(&parse(source)?)
                .unwrap_err()
                .to_string();
            assert!(error.contains("exceeds u8::MAX"), "{error}");
        }
        Ok(())
    }

    #[test]
    fn prefix_stream_has_fixed_arity_and_explicit_synthetic_tokens() -> Result<()> {
        use FlatPredicateToken as T;
        let expression = parse("adjustedPayloadLength >= 0x11 && !(hasTransactionID == NO)")?;
        let mut actual = Vec::new();
        expression
            .visit_prefix(&mut |token| {
                actual.push(token);
                Ok::<_, ()>(())
            })
            .unwrap();
        assert_eq!(
            actual,
            [
                T::And,
                T::Ge,
                T::AdjustedPayloadLength,
                T::Integer(17),
                T::Not,
                T::Eq,
                T::HasTransactionId,
                T::Bool(false),
            ]
        );
        Ok(())
    }
}
