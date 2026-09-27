#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]

use serde::{Deserialize, Serialize};

pub const CUSTOMER_REALM: &str = "ores-chat-customer";
pub const CUSTOMER_API_AUDIENCE: &str = "ores-chat-api";
pub const ADMIN_REALM: &str = "ores-chat-admin";
pub const ADMIN_API_AUDIENCE: &str = "ores-chat-admin-api";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatSurface {
    Public,
    User,
    Admin,
}

impl ChatSurface {
    #[must_use]
    pub const fn required_realm(self) -> Option<&'static str> {
        return match self {
            Self::Public => None,
            Self::User => Some(CUSTOMER_REALM),
            Self::Admin => Some(ADMIN_REALM),
        };
    }

    #[must_use]
    pub const fn required_token_audience(self) -> Option<&'static str> {
        return match self {
            Self::Public => None,
            Self::User => Some(CUSTOMER_API_AUDIENCE),
            Self::Admin => Some(ADMIN_API_AUDIENCE),
        };
    }
}
