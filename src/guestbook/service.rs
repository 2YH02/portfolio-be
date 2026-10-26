use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::db::DbPool;
use crate::errors::ServiceError;
use crate::guestbook::dto::{
    CreateGuestbookEntry, GuestbookEntryResponse, GuestbookModerationEntry,
};

const MAX_NICKNAME_CHARS: usize = 16;
const MAX_MESSAGE_CHARS: usize = 120;
const MAX_SCOPE_CHARS: usize = 64;
const RATE_LIMIT_SECONDS: i64 = 30;
const PUBLIC_LIMIT: i64 = 30;

#[derive(Debug, PartialEq, Eq)]
enum Moderation {
    Visible,
    Pending(&'static str),
    Blocked(&'static str),
}

pub fn hash_client_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"castmorrow-guestbook-v1:");
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn validate_scope(scope: &str) -> Result<String, ServiceError> {
    let scope = scope.trim().to_ascii_lowercase();
    if scope.is_empty()
        || scope.chars().count() > MAX_SCOPE_CHARS
        || !scope
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(ServiceError::BadRequest("잘못된 방명록 구분입니다".into()));
    }
    Ok(scope)
}

pub fn normalize_for_moderation(value: &str) -> String {
    value
        .nfkc()
        .flat_map(char::to_lowercase)
        .map(map_leetspeak)
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

fn map_leetspeak(ch: char) -> char {
    match ch {
        '0' => 'o',
        '1' => 'i',
        '3' => 'e',
        '4' => 'a',
        '5' => 's',
        '7' => 't',
        '@' => 'a',
        '$' => 's',
        _ => ch,
    }
}

fn moderate(nickname: &str, message: &str) -> Moderation {
    let text = normalize_for_moderation(&format!("{nickname} {message}"));
    const BLOCKED: &[(&str, &str)] = &[
        ("fuck", "profanity"),
        ("shit", "profanity"),
        ("bitch", "profanity"),
        ("asshole", "profanity"),
        ("cunt", "profanity"),
        ("dick", "profanity"),
        ("씨발", "profanity"),
        ("시발", "profanity"),
        ("병신", "profanity"),
        ("개새끼", "profanity"),
        ("존나", "profanity"),
        ("ㅅㅂ", "profanity"),
        ("ㅂㅅ", "profanity"),
        ("sex", "sexual"),
        ("porn", "sexual"),
        ("nude", "sexual"),
        ("xxx", "sexual"),
        ("섹스", "sexual"),
        ("야동", "sexual"),
        ("성관계", "sexual"),
        ("정치", "politics"),
        ("정당", "politics"),
        ("대통령", "politics"),
        ("국회", "politics"),
        ("선거", "politics"),
        ("투표", "politics"),
        ("후보", "politics"),
        ("politic", "politics"),
        ("election", "politics"),
        ("president", "politics"),
        ("government", "politics"),
        ("democrat", "politics"),
        ("republican", "politics"),
    ];
    if let Some((_, reason)) = BLOCKED.iter().find(|(word, _)| text.contains(word)) {
        return Moderation::Blocked(reason);
    }

    let compact = message
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<Vec<_>>();
    if compact.len() >= 8
        && compact
            .windows(8)
            .any(|window| window.iter().all(|ch| *ch == window[0]))
    {
        return Moderation::Pending("repeated");
    }
    let lowered = message.to_ascii_lowercase();
    if lowered.contains("http://") || lowered.contains("https://") || lowered.contains("www.") {
        return Moderation::Pending("link");
    }

    Moderation::Visible
}

fn validate_entry(dto: &CreateGuestbookEntry) -> Result<(&str, &str), ServiceError> {
    let nickname = dto.nickname.trim();
    let message = dto.message.trim();
    if nickname.is_empty() || nickname.chars().count() > MAX_NICKNAME_CHARS {
        return Err(ServiceError::BadRequest(
            "닉네임은 1~16자로 입력해주세요".into(),
        ));
    }
    if message.is_empty() || message.chars().count() > MAX_MESSAGE_CHARS {
        return Err(ServiceError::BadRequest(
            "내용은 1~120자로 입력해주세요".into(),
        ));
    }
    if !dto.website.trim().is_empty() {
        return Err(ServiceError::BadRequest("내용을 등록할 수 없습니다".into()));
    }
    Ok((nickname, message))
}

pub async fn list_visible(
    pool: &DbPool,
    scope: &str,
) -> Result<Vec<GuestbookEntryResponse>, ServiceError> {
    let client = pool.get().await?;
    let stmt = client
        .prepare_cached(
            "SELECT id, scope, nickname, message, created_at
             FROM guestbook_entries
             WHERE scope = $1 AND status = 'visible'
             ORDER BY created_at DESC, id DESC
             LIMIT $2",
        )
        .await?;
    let rows = client.query(&stmt, &[&scope, &PUBLIC_LIMIT]).await?;
    Ok(rows.into_iter().map(public_row).collect())
}

pub async fn create(
    pool: &DbPool,
    scope: &str,
    dto: CreateGuestbookEntry,
    client_hash: &str,
) -> Result<(GuestbookEntryResponse, bool), ServiceError> {
    let (nickname, message) = validate_entry(&dto)?;
    let moderation = moderate(nickname, message);
    let (status, reason) = match moderation {
        Moderation::Blocked(reason) => {
            return Err(ServiceError::BadRequest(format!(
                "허용되지 않는 내용입니다: {reason}"
            )));
        }
        Moderation::Pending(reason) => ("pending", Some(reason)),
        Moderation::Visible => ("visible", None),
    };

    let client = pool.get().await?;
    let limit_stmt = client
        .prepare_cached(
            "SELECT last_submitted_at
             FROM guestbook_rate_limits
             WHERE client_hash = $1
               AND last_submitted_at > NOW() - ($2 * INTERVAL '1 second')",
        )
        .await?;
    if client
        .query_opt(&limit_stmt, &[&client_hash, &RATE_LIMIT_SECONDS])
        .await?
        .is_some()
    {
        return Err(ServiceError::RateLimited);
    }

    let rate_stmt = client
        .prepare_cached(
            "INSERT INTO guestbook_rate_limits (client_hash, last_submitted_at)
             VALUES ($1, NOW())
             ON CONFLICT (client_hash)
             DO UPDATE SET last_submitted_at = NOW()",
        )
        .await?;
    client.execute(&rate_stmt, &[&client_hash]).await?;

    let stmt = client
        .prepare_cached(
            "INSERT INTO guestbook_entries
               (scope, nickname, message, status, moderation_reason, client_hash)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, scope, nickname, message, created_at",
        )
        .await?;
    let row = client
        .query_one(
            &stmt,
            &[&scope, &nickname, &message, &status, &reason, &client_hash],
        )
        .await?;
    Ok((public_row(row), status == "visible"))
}

pub async fn list_moderation(
    pool: &DbPool,
    status: Option<&str>,
) -> Result<Vec<GuestbookModerationEntry>, ServiceError> {
    let client = pool.get().await?;
    let rows = if let Some(status) = status {
        let stmt = client
            .prepare_cached(
                "SELECT id, scope, nickname, message, status, moderation_reason, created_at
                 FROM guestbook_entries WHERE status = $1
                 ORDER BY created_at ASC, id ASC LIMIT 100",
            )
            .await?;
        client.query(&stmt, &[&status]).await?
    } else {
        let stmt = client
            .prepare_cached(
                "SELECT id, scope, nickname, message, status, moderation_reason, created_at
                 FROM guestbook_entries
                 WHERE status IN ('pending', 'blocked')
                 ORDER BY created_at ASC, id ASC LIMIT 100",
            )
            .await?;
        client.query(&stmt, &[]).await?
    };
    Ok(rows
        .into_iter()
        .map(|row| GuestbookModerationEntry {
            id: row.get("id"),
            scope: row.get("scope"),
            nickname: row.get("nickname"),
            message: row.get("message"),
            status: row.get("status"),
            moderation_reason: row.get("moderation_reason"),
            created_at: row.get("created_at"),
        })
        .collect())
}

pub async fn moderate_entry(
    pool: &DbPool,
    id: i32,
    status: &str,
    reason: Option<&str>,
) -> Result<(), ServiceError> {
    if !matches!(status, "pending" | "visible" | "blocked" | "deleted") {
        return Err(ServiceError::BadRequest("잘못된 검수 상태입니다".into()));
    }
    let client = pool.get().await?;
    let stmt = client
        .prepare_cached(
            "UPDATE guestbook_entries
             SET status = $1, moderation_reason = $2
             WHERE id = $3",
        )
        .await?;
    let updated = client.execute(&stmt, &[&status, &reason, &id]).await?;
    if updated == 0 {
        return Err(ServiceError::NotFound);
    }
    Ok(())
}

pub async fn delete(pool: &DbPool, id: i32) -> Result<(), ServiceError> {
    let client = pool.get().await?;
    let stmt = client
        .prepare_cached("UPDATE guestbook_entries SET status = 'deleted' WHERE id = $1")
        .await?;
    if client.execute(&stmt, &[&id]).await? == 0 {
        return Err(ServiceError::NotFound);
    }
    Ok(())
}

fn public_row(row: tokio_postgres::Row) -> GuestbookEntryResponse {
    GuestbookEntryResponse {
        id: row.get("id"),
        scope: row.get("scope"),
        nickname: row.get("nickname"),
        message: row.get("message"),
        created_at: row.get("created_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::{moderate, normalize_for_moderation, validate_scope, Moderation};

    #[test]
    fn normalizes_unicode_and_common_leetspeak() {
        assert_eq!(normalize_for_moderation("Ｆｕｃｋ 5hit"), "fuckshit");
    }

    #[test]
    fn blocks_profanity_sexual_and_political_content() {
        assert_eq!(moderate("낚시꾼", "fuck"), Moderation::Blocked("profanity"));
        assert_eq!(
            moderate("낚시꾼", "섹스 이야기"),
            Moderation::Blocked("sexual")
        );
        assert_eq!(
            moderate("낚시꾼", "선거 이야기"),
            Moderation::Blocked("politics")
        );
    }

    #[test]
    fn puts_links_and_repeated_text_in_review() {
        assert_eq!(
            moderate("낚시꾼", "https://example.com"),
            Moderation::Pending("link")
        );
        assert_eq!(
            moderate("낚시꾼", "aaaaaaaaaa"),
            Moderation::Pending("repeated")
        );
    }

    #[test]
    fn accepts_scope_slugs_only() {
        assert_eq!(validate_scope("Castmorrow").unwrap(), "castmorrow");
        assert!(validate_scope("bad scope").is_err());
    }
}
