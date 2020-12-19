//! # Token module
//!
//! Functions to read/write token from/to the database. A token is any string associated with a key.
//!
//! Tokens are used in countermitm verification protocols.

use deltachat_derive::*;

use crate::chat::ChatId;
use crate::context::Context;
use crate::dc_tools::*;

/// Token namespace
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    FromPrimitive,
    ToPrimitive,
    ToSql,
    FromSql,
    sqlx::Type,
)]
#[repr(i32)]
pub enum Namespace {
    Unknown = 0,
    Auth = 110,
    InviteNumber = 100,
}

impl Default for Namespace {
    fn default() -> Self {
        Namespace::Unknown
    }
}

/// Creates a new token and saves it into the database.
/// Returns created token.
pub async fn save(context: &Context, namespace: Namespace, foreign_id: ChatId) -> String {
    // foreign_id may be 0
    let token = dc_create_id();
    context
        .sql
        .execute(
            sqlx::query(
                "INSERT INTO tokens (namespc, foreign_id, token, timestamp) VALUES (?, ?, ?, ?);",
            )
            .bind(namespace)
            .bind(foreign_id)
            .bind(&token)
            .bind(time()),
        )
        .await
        .ok();
    token
}

pub async fn lookup(
    context: &Context,
    namespace: Namespace,
    foreign_id: ChatId,
) -> crate::sql::Result<Option<String>> {
    let token = context
        .sql
        .query_get_value::<String>(
            "SELECT token FROM tokens WHERE namespc=? AND foreign_id=?;",
            paramsv![namespace, foreign_id],
        )
        .await?;
    Ok(token)
}

pub async fn lookup_or_new(context: &Context, namespace: Namespace, foreign_id: ChatId) -> String {
    if let Ok(Some(token)) = lookup(context, namespace, foreign_id).await {
        return token;
    }

    save(context, namespace, foreign_id).await
}

pub async fn exists(context: &Context, namespace: Namespace, token: &str) -> bool {
    context
        .sql
        .exists(
            sqlx::query("SELECT COUNT(*) FROM tokens WHERE namespc=? AND token=?;")
                .bind(namespace)
                .bind(token),
        )
        .await
        .unwrap_or_default()
}
