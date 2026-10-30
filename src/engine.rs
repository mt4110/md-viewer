//! The only connection to the real content inspector. No position inference.
use leak_fence_core::{Context, Cost, Denied, Policy};
use serde::{Deserialize, Serialize};

pub const ENGINE_REV: &str = "6a0f416e23da5e0cff5890fd05b6b1d9a4162393";
pub const ENGINE_REPO: &str = "https://github.com/mt4110/leak-fence";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialCode {
    Configuration,
    Unauthorized,
    ByteLimit,
    RecordLimit,
    InvalidJson,
    TenantMismatch,
    ObjectUnauthorized,
    UnapprovedField,
    NestedValue,
}

impl From<Denied> for DenialCode {
    fn from(reason: Denied) -> Self {
        // Exhaustive: an upstream variant addition must be reviewed.
        match reason {
            Denied::Configuration => Self::Configuration,
            Denied::Unauthorized => Self::Unauthorized,
            Denied::ByteLimit => Self::ByteLimit,
            Denied::RecordLimit => Self::RecordLimit,
            Denied::InvalidJson => Self::InvalidJson,
            Denied::TenantMismatch => Self::TenantMismatch,
            Denied::ObjectUnauthorized => Self::ObjectUnauthorized,
            Denied::UnapprovedField => Self::UnapprovedField,
            Denied::NestedValue => Self::NestedValue,
        }
    }
}

impl DenialCode {
    pub fn code(self) -> &'static str {
        match self {
            Self::Configuration => Denied::Configuration,
            Self::Unauthorized => Denied::Unauthorized,
            Self::ByteLimit => Denied::ByteLimit,
            Self::RecordLimit => Denied::RecordLimit,
            Self::InvalidJson => Denied::InvalidJson,
            Self::TenantMismatch => Denied::TenantMismatch,
            Self::ObjectUnauthorized => Denied::ObjectUnauthorized,
            Self::UnapprovedField => Denied::UnapprovedField,
            Self::NestedValue => Denied::NestedValue,
        }
        .code()
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "content_check", rename_all = "snake_case")]
pub enum ContentCheck {
    Passed { cost: Cost },
    Denied { reason: DenialCode },
}

impl ContentCheck {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Passed { .. } => "passed",
            Self::Denied { reason } => reason.code(),
        }
    }
}

pub fn evaluate_case(body: &[u8], policy: &Policy, context: &Context) -> ContentCheck {
    match leak_fence_core::inspect(body, policy, context) {
        Ok(cost) => ContentCheck::Passed { cost },
        Err(reason) => ContentCheck::Denied {
            reason: reason.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_reasons_are_the_actual_upstream_codes() {
        for reason in [
            Denied::Configuration,
            Denied::Unauthorized,
            Denied::ByteLimit,
            Denied::RecordLimit,
            Denied::InvalidJson,
            Denied::TenantMismatch,
            Denied::ObjectUnauthorized,
            Denied::UnapprovedField,
            Denied::NestedValue,
        ] {
            let code = DenialCode::from(reason);
            assert_eq!(code.code(), reason.code());
            assert_eq!(serde_json::to_value(code).unwrap(), reason.code());
        }
    }
}
