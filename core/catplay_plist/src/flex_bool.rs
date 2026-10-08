use alloc::string::String;
use core::ops::Deref;

/// Some CarPlay HUs send improper booleans that violate the spec, for example: `"oemIconVisible": String("1")`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FlexBool {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl Default for FlexBool {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl FlexBool {
    #[inline(always)]
    pub fn to_bool(&self) -> bool {
        match self {
            FlexBool::Bool(b) => *b,
            FlexBool::Int(i) => *i != 0,
            FlexBool::Str(s) => {
                matches!(s.as_str(), "1" | "true" | "TRUE" | "True")
            }
        }
    }
}

impl From<bool> for FlexBool {
    fn from(value: bool) -> Self {
        FlexBool::Bool(value)
    }
}

impl From<FlexBool> for bool {
    fn from(val: FlexBool) -> Self {
        val.to_bool()
    }
}

impl Deref for FlexBool {
    type Target = bool;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            FlexBool::Bool(b) => b,
            _ => {
                if self.to_bool() {
                    &true
                } else {
                    &false
                }
            }
        }
    }
}
