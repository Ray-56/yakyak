/// PostgreSQL implementation of VoicemailRepository
use crate::domain::voicemail::{VoicemailMailbox, VoicemailMessage, VoicemailRepository, VoicemailStatus};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::{debug, error};
use uuid::Uuid;

pub struct PgVoicemailRepository {
    pool: PgPool,
}

impl PgVoicemailRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VoicemailRepository for PgVoicemailRepository {
    async fn create_message(&self, message: VoicemailMessage) -> Result<VoicemailMessage, String> {
        let status_str = format!("{:?}", message.status);

        let result = sqlx::query(
            r#"
            INSERT INTO voicemail_messages
            (id, mailbox_id, caller, caller_name, duration_seconds, audio_file_path, audio_format,
             status, created_at, read_at, saved_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(message.id)
        .bind(&message.mailbox_id)
        .bind(&message.caller)
        .bind(message.caller_name.as_ref())
        .bind(message.duration_seconds as i32)
        .bind(&message.audio_file_path)
        .bind(&message.audio_format)
        .bind(&status_str)
        .bind(message.created_at)
        .bind(message.read_at)
        .bind(message.saved_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Created voicemail message: {}", message.id);
                Ok(message)
            }
            Err(e) => {
                error!("Failed to create voicemail message: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_message(&self, id: Uuid) -> Result<Option<VoicemailMessage>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, mailbox_id, caller, caller_name, duration_seconds, audio_file_path,
                   audio_format, status, created_at, read_at, saved_at
            FROM voicemail_messages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "New" => VoicemailStatus::New,
                    "Read" => VoicemailStatus::Read,
                    "Saved" => VoicemailStatus::Saved,
                    "Deleted" => VoicemailStatus::Deleted,
                    _ => VoicemailStatus::New,
                };

                let message = VoicemailMessage {
                    id: row.get("id"),
                    mailbox_id: row.get("mailbox_id"),
                    caller: row.get("caller"),
                    caller_name: row.get("caller_name"),
                    duration_seconds: row.get::<i32, _>("duration_seconds") as u32,
                    audio_file_path: row.get("audio_file_path"),
                    audio_format: row.get("audio_format"),
                    status,
                    created_at: row.get("created_at"),
                    read_at: row.get("read_at"),
                    saved_at: row.get("saved_at"),
                };

                Ok(Some(message))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get voicemail message: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn list_messages(
        &self,
        mailbox_id: &str,
        status: Option<VoicemailStatus>,
    ) -> Result<Vec<VoicemailMessage>, String> {
        let query = if let Some(status) = status {
            let status_str = format!("{:?}", status);
            sqlx::query(
                r#"
                SELECT id, mailbox_id, caller, caller_name, duration_seconds, audio_file_path,
                       audio_format, status, created_at, read_at, saved_at
                FROM voicemail_messages
                WHERE mailbox_id = $1 AND status = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(mailbox_id)
            .bind(&status_str)
        } else {
            sqlx::query(
                r#"
                SELECT id, mailbox_id, caller, caller_name, duration_seconds, audio_file_path,
                       audio_format, status, created_at, read_at, saved_at
                FROM voicemail_messages
                WHERE mailbox_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(mailbox_id)
        };

        let result = query.fetch_all(&self.pool).await;

        match result {
            Ok(rows) => {
                let messages: Vec<VoicemailMessage> = rows
                    .into_iter()
                    .map(|row| {
                        let status_str: String = row.get("status");
                        let status = match status_str.as_str() {
                            "New" => VoicemailStatus::New,
                            "Read" => VoicemailStatus::Read,
                            "Saved" => VoicemailStatus::Saved,
                            "Deleted" => VoicemailStatus::Deleted,
                            _ => VoicemailStatus::New,
                        };

                        VoicemailMessage {
                            id: row.get("id"),
                            mailbox_id: row.get("mailbox_id"),
                            caller: row.get("caller"),
                            caller_name: row.get("caller_name"),
                            duration_seconds: row.get::<i32, _>("duration_seconds") as u32,
                            audio_file_path: row.get("audio_file_path"),
                            audio_format: row.get("audio_format"),
                            status,
                            created_at: row.get("created_at"),
                            read_at: row.get("read_at"),
                            saved_at: row.get("saved_at"),
                        }
                    })
                    .collect();

                Ok(messages)
            }
            Err(e) => {
                error!("Failed to list voicemail messages: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_message_status(&self, id: Uuid, status: VoicemailStatus) -> Result<(), String> {
        let status_str = format!("{:?}", status);
        let now = chrono::Utc::now();

        // Update read_at or saved_at based on status
        let result = match status {
            VoicemailStatus::Read => {
                sqlx::query(
                    r#"
                    UPDATE voicemail_messages
                    SET status = $2, read_at = $3
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&status_str)
                .bind(now)
                .execute(&self.pool)
                .await
            }
            VoicemailStatus::Saved => {
                sqlx::query(
                    r#"
                    UPDATE voicemail_messages
                    SET status = $2, saved_at = $3
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&status_str)
                .bind(now)
                .execute(&self.pool)
                .await
            }
            _ => {
                sqlx::query(
                    r#"
                    UPDATE voicemail_messages
                    SET status = $2
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&status_str)
                .execute(&self.pool)
                .await
            }
        };

        match result {
            Ok(_) => {
                debug!("Updated voicemail message status: {}", id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update voicemail message status: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn delete_message(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM voicemail_messages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Deleted voicemail message: {}", id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete voicemail message: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn count_messages(
        &self,
        mailbox_id: &str,
        status: Option<VoicemailStatus>,
    ) -> Result<u32, String> {
        let query = if let Some(status) = status {
            let status_str = format!("{:?}", status);
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM voicemail_messages WHERE mailbox_id = $1 AND status = $2",
            )
            .bind(mailbox_id)
            .bind(&status_str)
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM voicemail_messages WHERE mailbox_id = $1",
            )
            .bind(mailbox_id)
        };

        let result = query.fetch_one(&self.pool).await;

        match result {
            Ok(count) => Ok(count as u32),
            Err(e) => {
                error!("Failed to count voicemail messages: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_mailbox(&self, mailbox_id: &str) -> Result<Option<VoicemailMailbox>, String> {
        let result = sqlx::query(
            r#"
            SELECT mailbox_id, user_id, pin, greeting_file, max_message_duration,
                   max_messages, email_notification, email_address, created_at, updated_at
            FROM voicemail_mailboxes
            WHERE mailbox_id = $1
            "#,
        )
        .bind(mailbox_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let mailbox = VoicemailMailbox {
                    mailbox_id: row.get("mailbox_id"),
                    user_id: row.get("user_id"),
                    pin: row.get("pin"),
                    greeting_file: row.get("greeting_file"),
                    max_message_duration: row.get::<i32, _>("max_message_duration") as u32,
                    max_messages: row.get::<i32, _>("max_messages") as u32,
                    email_notification: row.get("email_notification"),
                    email_address: row.get("email_address"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };

                Ok(Some(mailbox))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get voicemail mailbox: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn save_mailbox(&self, mailbox: VoicemailMailbox) -> Result<VoicemailMailbox, String> {
        // Try update first, then insert if it doesn't exist
        let update_result = sqlx::query(
            r#"
            UPDATE voicemail_mailboxes
            SET pin = $2, greeting_file = $3, max_message_duration = $4,
                max_messages = $5, email_notification = $6, email_address = $7, updated_at = $8
            WHERE mailbox_id = $1
            "#,
        )
        .bind(&mailbox.mailbox_id)
        .bind(mailbox.pin.as_ref())
        .bind(mailbox.greeting_file.as_ref())
        .bind(mailbox.max_message_duration as i32)
        .bind(mailbox.max_messages as i32)
        .bind(mailbox.email_notification)
        .bind(mailbox.email_address.as_ref())
        .bind(mailbox.updated_at)
        .execute(&self.pool)
        .await;

        match update_result {
            Ok(result) if result.rows_affected() > 0 => {
                debug!("Updated voicemail mailbox: {}", mailbox.mailbox_id);
                Ok(mailbox)
            }
            _ => {
                // Insert new mailbox
                let insert_result = sqlx::query(
                    r#"
                    INSERT INTO voicemail_mailboxes
                    (mailbox_id, user_id, pin, greeting_file, max_message_duration,
                     max_messages, email_notification, email_address, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    "#,
                )
                .bind(&mailbox.mailbox_id)
                .bind(mailbox.user_id)
                .bind(mailbox.pin.as_ref())
                .bind(mailbox.greeting_file.as_ref())
                .bind(mailbox.max_message_duration as i32)
                .bind(mailbox.max_messages as i32)
                .bind(mailbox.email_notification)
                .bind(mailbox.email_address.as_ref())
                .bind(mailbox.created_at)
                .bind(mailbox.updated_at)
                .execute(&self.pool)
                .await;

                match insert_result {
                    Ok(_) => {
                        debug!("Created voicemail mailbox: {}", mailbox.mailbox_id);
                        Ok(mailbox)
                    }
                    Err(e) => {
                        error!("Failed to save voicemail mailbox: {}", e);
                        Err(format!("Database error: {}", e))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL database with migrations applied
    // Run with: cargo test --features postgres voicemail_repository

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://yakyak:password@localhost/yakyak_test".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_and_get_mailbox() {
        let pool = setup_test_pool().await;
        let repo = PgVoicemailRepository::new(pool);

        let mailbox = VoicemailMailbox::new("alice".to_string(), 1);
        repo.save_mailbox(mailbox.clone()).await.unwrap();

        let retrieved = repo.get_mailbox("alice").await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_mailbox = retrieved.unwrap();
        assert_eq!(retrieved_mailbox.mailbox_id, "alice");
        assert_eq!(retrieved_mailbox.user_id, 1);
        assert_eq!(retrieved_mailbox.max_message_duration, 180);
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_and_list_messages() {
        let pool = setup_test_pool().await;
        let repo = PgVoicemailRepository::new(pool);

        // Create mailbox first
        let mailbox = VoicemailMailbox::new("bob".to_string(), 2);
        repo.save_mailbox(mailbox).await.unwrap();

        // Create messages
        let message1 = VoicemailMessage::new(
            "bob".to_string(),
            "sip:alice@example.com".to_string(),
            Some("Alice".to_string()),
            30,
            "/var/voicemail/bob/msg001.wav".to_string(),
            "wav".to_string(),
        );

        let message2 = VoicemailMessage::new(
            "bob".to_string(),
            "sip:charlie@example.com".to_string(),
            None,
            45,
            "/var/voicemail/bob/msg002.wav".to_string(),
            "wav".to_string(),
        );

        repo.create_message(message1.clone()).await.unwrap();
        repo.create_message(message2.clone()).await.unwrap();

        // List all messages
        let messages = repo.list_messages("bob", None).await.unwrap();
        assert_eq!(messages.len(), 2);

        // List new messages only
        let new_messages = repo
            .list_messages("bob", Some(VoicemailStatus::New))
            .await
            .unwrap();
        assert_eq!(new_messages.len(), 2);

        // Count messages
        let count = repo.count_messages("bob", None).await.unwrap();
        assert_eq!(count, 2);

        // Cleanup
        repo.delete_message(message1.id).await.unwrap();
        repo.delete_message(message2.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_update_message_status() {
        let pool = setup_test_pool().await;
        let repo = PgVoicemailRepository::new(pool);

        // Create mailbox and message
        let mailbox = VoicemailMailbox::new("charlie".to_string(), 3);
        repo.save_mailbox(mailbox).await.unwrap();

        let message = VoicemailMessage::new(
            "charlie".to_string(),
            "sip:dave@example.com".to_string(),
            None,
            60,
            "/var/voicemail/charlie/msg001.wav".to_string(),
            "wav".to_string(),
        );
        let message_id = message.id;

        repo.create_message(message).await.unwrap();

        // Update status to Read
        repo.update_message_status(message_id, VoicemailStatus::Read)
            .await
            .unwrap();

        let retrieved = repo.get_message(message_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, VoicemailStatus::Read);
        assert!(retrieved.read_at.is_some());

        // Cleanup
        repo.delete_message(message_id).await.unwrap();
    }
}
