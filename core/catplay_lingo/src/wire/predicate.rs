use super::{DecodeContext, EncodeContext};

/// Local wire-layer copy of codegen's flat prefix predicate token vocabulary.
/// Keep this independent from `codegen::ir`: generated schemas can use these
/// tokens without making the wire layer depend on the generator's IR.
#[repr(C)]
#[derive(Clone, Copy)]
union PredicateTokenPayload {
    bits: u8,
}

/// Compact prefix-predicate instruction: one tag byte and an unaligned u64.
/// The payload is canonical for each tag, so safe constructors always produce
/// a valid token and the wire interpreter can decode it without pointer data.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FlatPredicateToken {
    tag: u8,
    payload: PredicateTokenPayload,
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
impl FlatPredicateToken {
    const fn new(tag: u8, bits: u8) -> Self {
        Self {
            tag,
            payload: PredicateTokenPayload { bits },
        }
    }

    fn decoded(self) -> (u8, u8) {
        // SAFETY: the packed payload is potentially unaligned. Reading as u64 is
        // valid for all bit patterns, and every constructor initializes it.
        let payload = unsafe { core::ptr::addr_of!(self.payload).read_unaligned() };
        (self.tag, unsafe { payload.bits })
    }

    #[allow(non_snake_case)]
    pub const fn Field(value: u8) -> Self {
        Self::new(0, value)
    }
    #[allow(non_snake_case)]
    pub const fn Integer(value: u8) -> Self {
        Self::new(4, value)
    }
    #[allow(non_snake_case)]
    pub const fn Bool(value: bool) -> Self {
        Self::new(5, value as u8)
    }

    #[allow(non_upper_case_globals)]
    pub const AdjustedPayloadLength: Self = Self::new(1, 0);
    #[allow(non_upper_case_globals)]
    pub const HasTransactionId: Self = Self::new(2, 0);
    #[allow(non_upper_case_globals)]
    pub const Encoding: Self = Self::new(3, 0);
    #[allow(non_upper_case_globals)]
    pub const Eq: Self = Self::new(6, 0);
    #[allow(non_upper_case_globals)]
    pub const Ne: Self = Self::new(7, 0);
    #[allow(non_upper_case_globals)]
    pub const Lt: Self = Self::new(8, 0);
    #[allow(non_upper_case_globals)]
    pub const Le: Self = Self::new(9, 0);
    #[allow(non_upper_case_globals)]
    pub const Gt: Self = Self::new(10, 0);
    #[allow(non_upper_case_globals)]
    pub const Ge: Self = Self::new(11, 0);
    #[allow(non_upper_case_globals)]
    pub const And: Self = Self::new(12, 0);
    #[allow(non_upper_case_globals)]
    pub const Or: Self = Self::new(13, 0);
    #[allow(non_upper_case_globals)]
    pub const Not: Self = Self::new(14, 0);
}

impl core::fmt::Debug for FlatPredicateToken {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (tag, bits) = (*self).decoded();
        match tag {
            0 => formatter.debug_tuple("Field").field(&bits).finish(),
            1 => formatter.write_str("AdjustedPayloadLength"),
            2 => formatter.write_str("HasTransactionId"),
            3 => formatter.write_str("Encoding"),
            4 => formatter.debug_tuple("Integer").field(&bits).finish(),
            5 => formatter.debug_tuple("Bool").field(&(bits != 0)).finish(),
            6 => formatter.write_str("Eq"),
            7 => formatter.write_str("Ne"),
            8 => formatter.write_str("Lt"),
            9 => formatter.write_str("Le"),
            10 => formatter.write_str("Gt"),
            11 => formatter.write_str("Ge"),
            12 => formatter.write_str("And"),
            13 => formatter.write_str("Or"),
            14 => formatter.write_str("Not"),
            _ => formatter.write_str("InvalidPredicateToken"),
        }
    }
}

impl PartialEq for FlatPredicateToken {
    fn eq(&self, other: &Self) -> bool {
        (*self).decoded() == (*other).decoded()
    }
}
impl Eq for FlatPredicateToken {}

/// A constant prefix program emitted from the schema predicate AST.
#[derive(Clone, Copy, Debug)]
pub struct PredicateProgram<'a> {
    tokens: &'a [FlatPredicateToken],
}

impl<'a> PredicateProgram<'a> {
    pub const fn new(tokens: &'a [FlatPredicateToken]) -> Self {
        Self { tokens }
    }

    pub fn eval_decode(&self, fields: &[Option<i128>; 128], ctx: &DecodeContext) -> bool {
        self.eval(fields, ctx.adjusted_payload_length, ctx.has_transaction_id, false)
    }

    pub fn eval_encode(&self, fields: &[Option<i128>; 128], ctx: &EncodeContext) -> bool {
        self.eval(fields, ctx.adjusted_payload_length, ctx.has_transaction_id, true)
    }

    fn eval(&self, fields: &[Option<i128>; 128], length: usize, has_tx: bool, encoding: bool) -> bool {
        let mut cursor = 0;
        let Some(value) = eval_node(self.tokens, &mut cursor, fields, length, has_tx, encoding) else {
            return false;
        };
        cursor == self.tokens.len() && value != 0
    }
}

fn eval_node(
    tokens: &[FlatPredicateToken],
    cursor: &mut usize,
    fields: &[Option<i128>; 128],
    length: usize,
    has_tx: bool,
    encoding: bool,
) -> Option<i128> {
    let token = *tokens.get(*cursor)?;
    *cursor += 1;
    let (tag, payload) = token.decoded();
    let boolean = |value: bool| Some(i128::from(value));
    match tag {
        0 => fields.get(payload as usize).copied().flatten(),
        1 => Some(length as i128),
        2 => Some(i128::from(has_tx)),
        3 => Some(i128::from(encoding)),
        4 => Some(i128::from(payload)),
        5 => Some(i128::from(payload != 0)),
        14 => boolean(!eval_node(tokens, cursor, fields, length, has_tx, encoding).is_some_and(|v| v != 0)),
        12 | 13 => {
            let left = eval_node(tokens, cursor, fields, length, has_tx, encoding).is_some_and(|v| v != 0);
            let right = eval_node(tokens, cursor, fields, length, has_tx, encoding).is_some_and(|v| v != 0);
            boolean(if tag == 12 { left && right } else { left || right })
        }
        6..=11 => {
            let left = eval_node(tokens, cursor, fields, length, has_tx, encoding);
            let right = eval_node(tokens, cursor, fields, length, has_tx, encoding);
            match (left, right) {
                (Some(left), Some(right)) => boolean(match tag {
                    6 => left == right,
                    7 => left != right,
                    8 => left < right,
                    9 => left <= right,
                    10 => left > right,
                    11 => left >= right,
                    _ => unreachable!(),
                }),
                _ => Some(0),
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{FlatPredicateToken as I, PredicateProgram};
    use crate::{DecodeContext, EncodeContext};

    #[test]
    fn evaluates_field_and_context_operands_from_const_program() {
        let tokens = [
            I::And,
            I::Eq,
            I::Field(3),
            I::Integer(2),
            I::Or,
            I::HasTransactionId,
            I::Gt,
            I::AdjustedPayloadLength,
            I::Integer(8),
        ];
        let program = PredicateProgram::new(&tokens);
        let mut fields = [None; 128];
        fields[3] = Some(2);
        assert!(program.eval_decode(
            &fields,
            &DecodeContext {
                adjusted_payload_length: 9,
                has_transaction_id: false
            }
        ));
        assert!(!program.eval_decode(
            &fields,
            &DecodeContext {
                adjusted_payload_length: 8,
                has_transaction_id: false
            }
        ));
        assert!(program.eval_decode(
            &fields,
            &DecodeContext {
                adjusted_payload_length: 0,
                has_transaction_id: true
            }
        ));
        fields[3] = None;
        assert!(!program.eval_decode(
            &fields,
            &DecodeContext {
                adjusted_payload_length: 9,
                has_transaction_id: true
            }
        ));
    }

    #[test]
    fn encoding_instruction_can_bypass_length_predicates() {
        let tokens = [I::Or, I::Encoding, I::Gt, I::AdjustedPayloadLength, I::Integer(4)];
        let program = PredicateProgram::new(&tokens);
        let fields = [None; 128];
        assert!(program.eval_encode(&fields, &EncodeContext::default()));
        assert!(!program.eval_decode(&fields, &DecodeContext::default()));
    }

    #[test]
    fn evaluates_mixed_and_or_groups() {
        let tokens = [
            I::Or,
            I::And,
            I::Eq,
            I::Field(0),
            I::Integer(1),
            I::Eq,
            I::Field(1),
            I::Integer(1),
            I::And,
            I::Eq,
            I::Field(2),
            I::Integer(1),
            I::Eq,
            I::Field(3),
            I::Integer(1),
        ];
        let program = PredicateProgram::new(&tokens);
        let mut fields = [None; 128];
        fields[..4].copy_from_slice(&[Some(1), Some(1), Some(0), Some(1)]);
        assert!(program.eval_decode(&fields, &DecodeContext::default()));
        fields[0] = Some(0);
        assert!(!program.eval_decode(&fields, &DecodeContext::default()));
        fields[2] = Some(1);
        assert!(program.eval_decode(&fields, &DecodeContext::default()));
    }

    #[test]
    fn compact_tokens_use_two_bytes_and_round_trip_by_value() {
        let tokens = [
            I::Field(127),
            I::AdjustedPayloadLength,
            I::HasTransactionId,
            I::Encoding,
            I::Integer(u8::MAX),
            I::Bool(false),
            I::Bool(true),
            I::Eq,
            I::Ne,
            I::Lt,
            I::Le,
            I::Gt,
            I::Ge,
            I::And,
            I::Or,
            I::Not,
        ];
        assert_eq!(core::mem::size_of::<I>(), 2);
        assert_eq!(core::mem::align_of::<I>(), 1);
        assert_eq!(format!("{:?}", I::Integer(42)), "Integer(42)");
        assert_eq!(I::Integer(9), I::Integer(9));
        assert_ne!(I::Integer(9), I::Integer(10));
        assert_eq!(tokens[4], I::Integer(u8::MAX));
    }
}
