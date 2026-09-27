#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]

use ores_chat_trust_surface_proof::{
    ADMIN_API_AUDIENCE, ADMIN_REALM, CUSTOMER_API_AUDIENCE, CUSTOMER_REALM, ChatSurface,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

const PROTOCOL: &str = "ores.chat.trust_surface/v1";
const MODELS: [&str; 3] = [
    "PublicChatSurfacePolicy",
    "UserChatSurfacePolicy",
    "AdminChatSurfacePolicy",
];

fn proof_file(name: &str) -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust proof has a parent directory")
        .join(name);
}

fn typespec() -> String {
    return fs::read_to_string(proof_file("main.tsp")).expect("TypeSpec proof fixture must exist");
}

fn schema() -> Value {
    let source = fs::read_to_string(proof_file("authored.schema.json"))
        .expect("JSON Schema proof fixture must exist");
    return serde_json::from_str(&source).expect("JSON Schema proof fixture must parse");
}

fn block<'a>(source: &'a str, kind: &str, name: &str) -> &'a str {
    let marker = format!("{kind} {name} {{");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {kind} {name}"));
    let body_start = start + marker.len();
    let tail = &source[body_start..];
    let mut depth = 1_usize;
    for (offset, character) in tail.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &tail[..offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated {kind} {name}");
}

fn typespec_enum_values(source: &str) -> BTreeSet<String> {
    return block(source, "enum", "ChatSurface")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect();
}

fn typespec_literals(source: &str, model: &str) -> BTreeMap<String, String> {
    return block(source, "model", model)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, value) = line
                .trim_end_matches(';')
                .split_once(':')
                .unwrap_or_else(|| panic!("{model} field must declare a literal"));
            return (
                name.trim().to_owned(),
                value.trim().trim_matches('"').to_owned(),
            );
        })
        .collect();
}

fn schema_literals(schema: &Value, model: &str) -> BTreeMap<String, String> {
    return schema["$defs"][model]["properties"]
        .as_object()
        .unwrap_or_else(|| panic!("{model} must declare properties"))
        .iter()
        .map(|(name, property)| {
            return (
                name.clone(),
                property["const"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{model}.{name} must be a string const"))
                    .to_owned(),
            );
        })
        .collect();
}

#[test]
fn surface_enum_matches_both_authored_authorities() {
    let tsp = typespec();
    let schema = schema();
    let expected = BTreeSet::from(["admin".to_owned(), "public".to_owned(), "user".to_owned()]);
    let schema_values = schema["$defs"]["ChatSurface"]["enum"]
        .as_array()
        .expect("ChatSurface enum must exist")
        .iter()
        .map(|value| value.as_str().expect("enum value must be text").to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(typespec_enum_values(&tsp), expected);
    assert_eq!(schema_values, expected);
    assert_eq!(serde_json::to_value(ChatSurface::Public).unwrap(), "public");
    assert_eq!(serde_json::to_value(ChatSurface::User).unwrap(), "user");
    assert_eq!(serde_json::to_value(ChatSurface::Admin).unwrap(), "admin");
}

#[test]
fn authored_policy_literals_match_exactly() {
    let tsp = typespec();
    let schema = schema();
    for model in MODELS {
        let tsp_policy = typespec_literals(&tsp, model);
        let json_policy = schema_literals(&schema, model);
        assert_eq!(json_policy, tsp_policy, "{model} authority drift");
        assert_eq!(
            json_policy.get("protocol").map(String::as_str),
            Some(PROTOCOL)
        );
    }
}

#[test]
fn rust_semantics_match_public_customer_and_admin_policy() {
    let schema = schema();
    let public = schema_literals(&schema, "PublicChatSurfacePolicy");
    let user = schema_literals(&schema, "UserChatSurfacePolicy");
    let admin = schema_literals(&schema, "AdminChatSurfacePolicy");

    assert_eq!(ChatSurface::Public.required_realm(), None);
    assert_eq!(ChatSurface::Public.required_token_audience(), None);
    assert!(!public.contains_key("required_realm"));
    assert!(!public.contains_key("required_token_audience"));

    assert_eq!(ChatSurface::User.required_realm(), Some(CUSTOMER_REALM));
    assert_eq!(
        ChatSurface::User.required_token_audience(),
        Some(CUSTOMER_API_AUDIENCE)
    );
    assert_eq!(
        user.get("required_realm").map(String::as_str),
        Some(CUSTOMER_REALM)
    );
    assert_eq!(
        user.get("required_token_audience").map(String::as_str),
        Some(CUSTOMER_API_AUDIENCE)
    );

    assert_eq!(ChatSurface::Admin.required_realm(), Some(ADMIN_REALM));
    assert_eq!(
        ChatSurface::Admin.required_token_audience(),
        Some(ADMIN_API_AUDIENCE)
    );
    assert_eq!(
        admin.get("required_realm").map(String::as_str),
        Some(ADMIN_REALM)
    );
    assert_eq!(
        admin.get("required_token_audience").map(String::as_str),
        Some(ADMIN_API_AUDIENCE)
    );
}
