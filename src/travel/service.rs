use bcrypt::{DEFAULT_COST, hash, verify};
use std::collections::HashMap;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::db::DbPool;
use crate::errors::ServiceError;
use crate::travel::dto::{CreateTravelComment, TravelCommentResponse, TravelReactionResponse};
use crate::travel::model::{TravelComment, TravelReaction};

const MAX_AUTHOR_CHARS: usize = 50;
const MAX_CONTENT_CHARS: usize = 500;
const MAX_BCRYPT_PASSWORD_BYTES: usize = 72;

pub fn emoji_cookie_token(emoji: &str) -> Option<&'static str> {
    match emoji {
        "👍" => Some("thumbs_up"),
        "❤️" => Some("heart"),
        "😂" => Some("joy"),
        "🔥" => Some("fire"),
        "✈️" => Some("plane"),
        "🍜" => Some("ramen"),
        _ => None,
    }
}

pub async fn get_like_count(pool: &DbPool, slug: &str) -> Result<i32, ServiceError> {
    let client = pool.get().await?;

    let stmt = client
        .prepare_cached("SELECT like_count FROM travel_likes WHERE slug = $1")
        .await?;
    let row = client.query_opt(&stmt, &[&slug]).await?;

    Ok(row.map(|row| row.get(0)).unwrap_or(0))
}

pub async fn increment_like(pool: &DbPool, slug: &str) -> Result<i32, ServiceError> {
    let client = pool.get().await?;

    let stmt = client
        .prepare_cached(
            "INSERT INTO travel_likes (slug, like_count)
             VALUES ($1, 1)
             ON CONFLICT (slug)
             DO UPDATE SET like_count = travel_likes.like_count + 1
             RETURNING like_count",
        )
        .await?;
    let row = client.query_one(&stmt, &[&slug]).await?;

    Ok(row.get(0))
}

pub async fn decrement_like(pool: &DbPool, slug: &str) -> Result<i32, ServiceError> {
    let client = pool.get().await?;

    let stmt = client
        .prepare_cached(
            "UPDATE travel_likes
             SET like_count = GREATEST(like_count - 1, 0)
             WHERE slug = $1
             RETURNING like_count",
        )
        .await?;
    let row = client.query_opt(&stmt, &[&slug]).await?;

    Ok(row.map(|row| row.get(0)).unwrap_or(0))
}

pub async fn list_comments(
    pool: &DbPool,
    slug: &str,
) -> Result<Vec<TravelCommentResponse>, ServiceError> {
    let client = pool.get().await?;

    let stmt = client
        .prepare_cached(
            "SELECT id, author, content, created_at
             FROM travel_comments
             WHERE slug = $1
             ORDER BY created_at ASC, id ASC",
        )
        .await?;
    let rows = client.query(&stmt, &[&slug]).await?;

    let comments = rows
        .into_iter()
        .map(|row| TravelComment::from_row_ref(&row).map_err(ServiceError::from))
        .collect::<Result<Vec<_>, _>>()?;
    let comment_ids = comments
        .iter()
        .map(|comment| comment.id)
        .collect::<Vec<_>>();

    let mut reactions_by_comment: HashMap<i32, Vec<TravelReactionResponse>> = HashMap::new();
    if !comment_ids.is_empty() {
        let stmt = client
            .prepare_cached(
                "SELECT comment_id, emoji, count
                 FROM travel_reactions
                 WHERE comment_id = ANY($1)
                 ORDER BY id ASC",
            )
            .await?;
        let reaction_rows = client.query(&stmt, &[&comment_ids]).await?;

        for row in reaction_rows {
            let comment_id: i32 = row.get("comment_id");
            let emoji: String = row.get("emoji");
            let count: i32 = row.get("count");
            reactions_by_comment
                .entry(comment_id)
                .or_default()
                .push(TravelReactionResponse { emoji, count });
        }
    }

    Ok(comments
        .into_iter()
        .map(|comment| {
            let reactions = reactions_by_comment.remove(&comment.id).unwrap_or_default();
            TravelCommentResponse {
                id: comment.id,
                author: comment.author,
                content: comment.content,
                created_at: comment.created_at,
                reactions,
            }
        })
        .collect())
}

pub async fn create_comment(
    pool: &DbPool,
    slug: &str,
    dto: CreateTravelComment,
) -> Result<TravelCommentResponse, ServiceError> {
    let author = dto.author.trim();
    let password = dto.password.trim();
    let content = dto.content.trim();

    validate_comment(author, password, content)?;

    let password_hash = hash(password, DEFAULT_COST)
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let client = pool.get().await?;
    let stmt = client
        .prepare_cached(
            "INSERT INTO travel_comments (slug, author, password, content)
             VALUES ($1, $2, $3, $4)
             RETURNING id, author, content, created_at",
        )
        .await?;
    let row = client
        .query_one(&stmt, &[&slug, &author, &password_hash, &content])
        .await?;
    let comment = TravelComment::from_row_ref(&row)?;

    Ok(TravelCommentResponse {
        id: comment.id,
        author: comment.author,
        content: comment.content,
        created_at: comment.created_at,
        reactions: Vec::new(),
    })
}

pub async fn delete_comment(
    pool: &DbPool,
    comment_id: i32,
    password: &str,
) -> Result<(), ServiceError> {
    let password = password.trim();
    if password.is_empty() {
        return Err(ServiceError::BadRequest("비밀번호를 입력해주세요".into()));
    }

    let client = pool.get().await?;
    let stmt = client
        .prepare_cached("SELECT password FROM travel_comments WHERE id = $1")
        .await?;
    let row = client.query_opt(&stmt, &[&comment_id]).await?;
    let Some(row) = row else {
        return Err(ServiceError::NotFound);
    };

    let password_hash: String = row.get(0);
    let verified = verify(password, &password_hash)
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    if !verified {
        return Err(ServiceError::Unauthorized);
    }

    let stmt = client
        .prepare_cached("DELETE FROM travel_comments WHERE id = $1")
        .await?;
    client.execute(&stmt, &[&comment_id]).await?;

    Ok(())
}

pub async fn increment_reaction(
    pool: &DbPool,
    comment_id: i32,
    emoji: &str,
) -> Result<TravelReactionResponse, ServiceError> {
    if emoji_cookie_token(emoji).is_none() {
        return Err(ServiceError::BadRequest(
            "허용되지 않은 이모지입니다".into(),
        ));
    }

    let client = pool.get().await?;

    let exists_stmt = client
        .prepare_cached("SELECT 1 FROM travel_comments WHERE id = $1")
        .await?;
    if client
        .query_opt(&exists_stmt, &[&comment_id])
        .await?
        .is_none()
    {
        return Err(ServiceError::NotFound);
    }

    let stmt = client
        .prepare_cached(
            "INSERT INTO travel_reactions (comment_id, emoji, count)
             VALUES ($1, $2, 1)
             ON CONFLICT (comment_id, emoji)
             DO UPDATE SET count = travel_reactions.count + 1
             RETURNING emoji, count",
        )
        .await?;
    let row = client.query_one(&stmt, &[&comment_id, &emoji]).await?;
    let reaction = TravelReaction::from_row_ref(&row)?;

    Ok(TravelReactionResponse {
        emoji: reaction.emoji,
        count: reaction.count,
    })
}

fn validate_comment(author: &str, password: &str, content: &str) -> Result<(), ServiceError> {
    if author.is_empty() {
        return Err(ServiceError::BadRequest("작성자를 입력해주세요".into()));
    }
    if author.chars().count() > MAX_AUTHOR_CHARS {
        return Err(ServiceError::BadRequest(
            "작성자는 50자 이하로 입력해주세요".into(),
        ));
    }
    if password.is_empty() {
        return Err(ServiceError::BadRequest("비밀번호를 입력해주세요".into()));
    }
    if password.len() > MAX_BCRYPT_PASSWORD_BYTES {
        return Err(ServiceError::BadRequest(
            "비밀번호는 72바이트 이하로 입력해주세요".into(),
        ));
    }
    if content.is_empty() {
        return Err(ServiceError::BadRequest("댓글 내용을 입력해주세요".into()));
    }
    if content.chars().count() > MAX_CONTENT_CHARS {
        return Err(ServiceError::BadRequest(
            "댓글은 500자 이하로 입력해주세요".into(),
        ));
    }

    Ok(())
}
