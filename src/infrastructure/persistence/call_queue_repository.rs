/// PostgreSQL implementation of CallQueueRepository
use crate::domain::call_queue::{
    AgentStatus, CallQueue, CallQueueRepository, OverflowAction, QueueMember, QueueStrategy,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::time::Duration;
use tracing::{debug, error};
use uuid::Uuid;

pub struct PgCallQueueRepository {
    pool: PgPool,
}

impl PgCallQueueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CallQueueRepository for PgCallQueueRepository {
    async fn create_queue(&self, queue: CallQueue) -> Result<CallQueue, String> {
        let strategy_str = format!("{:?}", queue.strategy);
        let overflow_action_str = format!("{:?}", queue.overflow_action);

        let result = sqlx::query(
            r#"
            INSERT INTO call_queues
            (id, name, extension, strategy, max_wait_time_secs, max_queue_size, ring_timeout_secs,
             retry_delay_secs, max_retries, wrap_up_time_secs, announce_position, announce_wait_time,
             music_on_hold, periodic_announce, periodic_announce_frequency_secs,
             overflow_queue_id, overflow_action, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(queue.id)
        .bind(&queue.name)
        .bind(&queue.extension)
        .bind(&strategy_str)
        .bind(queue.max_wait_time.as_secs() as i64)
        .bind(queue.max_queue_size as i32)
        .bind(queue.ring_timeout.as_secs() as i64)
        .bind(queue.retry_delay.as_secs() as i64)
        .bind(queue.max_retries as i32)
        .bind(queue.wrap_up_time.as_secs() as i64)
        .bind(queue.announce_position)
        .bind(queue.announce_wait_time)
        .bind(queue.music_on_hold.as_ref())
        .bind(queue.periodic_announce.as_ref())
        .bind(queue.periodic_announce_frequency.as_secs() as i64)
        .bind(queue.overflow_queue_id)
        .bind(&overflow_action_str)
        .bind(queue.created_at)
        .bind(queue.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Created call queue: {}", queue.id);
                Ok(queue)
            }
            Err(e) => {
                error!("Failed to create call queue: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_queue(&self, queue_id: Uuid) -> Result<Option<CallQueue>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, extension, strategy, max_wait_time_secs, max_queue_size, ring_timeout_secs,
                   retry_delay_secs, max_retries, wrap_up_time_secs, announce_position, announce_wait_time,
                   music_on_hold, periodic_announce, periodic_announce_frequency_secs,
                   overflow_queue_id, overflow_action, created_at, updated_at
            FROM call_queues
            WHERE id = $1
            "#,
        )
        .bind(queue_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let strategy_str: String = row.get("strategy");
                let strategy = match strategy_str.as_str() {
                    "RingAll" => QueueStrategy::RingAll,
                    "Linear" => QueueStrategy::Linear,
                    "LeastRecent" => QueueStrategy::LeastRecent,
                    "FewestCalls" => QueueStrategy::FewestCalls,
                    "LeastTalkTime" => QueueStrategy::LeastTalkTime,
                    "Random" => QueueStrategy::Random,
                    "RoundRobin" => QueueStrategy::RoundRobin,
                    _ => QueueStrategy::RoundRobin,
                };

                let overflow_action_str: String = row.get("overflow_action");
                let overflow_action = match overflow_action_str.as_str() {
                    "Busy" => OverflowAction::Busy,
                    "Voicemail" => OverflowAction::Voicemail,
                    "ForwardToQueue" => OverflowAction::ForwardToQueue,
                    "ForwardToExtension" => OverflowAction::ForwardToExtension,
                    "Announcement" => OverflowAction::Announcement,
                    _ => OverflowAction::Busy,
                };

                let queue = CallQueue {
                    id: row.get("id"),
                    name: row.get("name"),
                    extension: row.get("extension"),
                    strategy,
                    max_wait_time: Duration::from_secs(row.get::<i64, _>("max_wait_time_secs") as u64),
                    max_queue_size: row.get::<i32, _>("max_queue_size") as usize,
                    ring_timeout: Duration::from_secs(row.get::<i64, _>("ring_timeout_secs") as u64),
                    retry_delay: Duration::from_secs(row.get::<i64, _>("retry_delay_secs") as u64),
                    max_retries: row.get::<i32, _>("max_retries") as u32,
                    wrap_up_time: Duration::from_secs(row.get::<i64, _>("wrap_up_time_secs") as u64),
                    announce_position: row.get("announce_position"),
                    announce_wait_time: row.get("announce_wait_time"),
                    music_on_hold: row.get("music_on_hold"),
                    periodic_announce: row.get("periodic_announce"),
                    periodic_announce_frequency: Duration::from_secs(
                        row.get::<i64, _>("periodic_announce_frequency_secs") as u64,
                    ),
                    overflow_queue_id: row.get("overflow_queue_id"),
                    overflow_action,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };

                Ok(Some(queue))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get call queue: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_queue_by_extension(&self, extension: &str) -> Result<Option<CallQueue>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, extension, strategy, max_wait_time_secs, max_queue_size, ring_timeout_secs,
                   retry_delay_secs, max_retries, wrap_up_time_secs, announce_position, announce_wait_time,
                   music_on_hold, periodic_announce, periodic_announce_frequency_secs,
                   overflow_queue_id, overflow_action, created_at, updated_at
            FROM call_queues
            WHERE extension = $1
            "#,
        )
        .bind(extension)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let strategy_str: String = row.get("strategy");
                let strategy = match strategy_str.as_str() {
                    "RingAll" => QueueStrategy::RingAll,
                    "Linear" => QueueStrategy::Linear,
                    "LeastRecent" => QueueStrategy::LeastRecent,
                    "FewestCalls" => QueueStrategy::FewestCalls,
                    "LeastTalkTime" => QueueStrategy::LeastTalkTime,
                    "Random" => QueueStrategy::Random,
                    "RoundRobin" => QueueStrategy::RoundRobin,
                    _ => QueueStrategy::RoundRobin,
                };

                let overflow_action_str: String = row.get("overflow_action");
                let overflow_action = match overflow_action_str.as_str() {
                    "Busy" => OverflowAction::Busy,
                    "Voicemail" => OverflowAction::Voicemail,
                    "ForwardToQueue" => OverflowAction::ForwardToQueue,
                    "ForwardToExtension" => OverflowAction::ForwardToExtension,
                    "Announcement" => OverflowAction::Announcement,
                    _ => OverflowAction::Busy,
                };

                let queue = CallQueue {
                    id: row.get("id"),
                    name: row.get("name"),
                    extension: row.get("extension"),
                    strategy,
                    max_wait_time: Duration::from_secs(row.get::<i64, _>("max_wait_time_secs") as u64),
                    max_queue_size: row.get::<i32, _>("max_queue_size") as usize,
                    ring_timeout: Duration::from_secs(row.get::<i64, _>("ring_timeout_secs") as u64),
                    retry_delay: Duration::from_secs(row.get::<i64, _>("retry_delay_secs") as u64),
                    max_retries: row.get::<i32, _>("max_retries") as u32,
                    wrap_up_time: Duration::from_secs(row.get::<i64, _>("wrap_up_time_secs") as u64),
                    announce_position: row.get("announce_position"),
                    announce_wait_time: row.get("announce_wait_time"),
                    music_on_hold: row.get("music_on_hold"),
                    periodic_announce: row.get("periodic_announce"),
                    periodic_announce_frequency: Duration::from_secs(
                        row.get::<i64, _>("periodic_announce_frequency_secs") as u64,
                    ),
                    overflow_queue_id: row.get("overflow_queue_id"),
                    overflow_action,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };

                Ok(Some(queue))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get call queue by extension: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_queue(&self, queue: &CallQueue) -> Result<(), String> {
        let strategy_str = format!("{:?}", queue.strategy);
        let overflow_action_str = format!("{:?}", queue.overflow_action);

        let result = sqlx::query(
            r#"
            UPDATE call_queues
            SET name = $2, extension = $3, strategy = $4, max_wait_time_secs = $5,
                max_queue_size = $6, ring_timeout_secs = $7, retry_delay_secs = $8,
                max_retries = $9, wrap_up_time_secs = $10, announce_position = $11,
                announce_wait_time = $12, music_on_hold = $13, periodic_announce = $14,
                periodic_announce_frequency_secs = $15, overflow_queue_id = $16,
                overflow_action = $17, updated_at = $18
            WHERE id = $1
            "#,
        )
        .bind(queue.id)
        .bind(&queue.name)
        .bind(&queue.extension)
        .bind(&strategy_str)
        .bind(queue.max_wait_time.as_secs() as i64)
        .bind(queue.max_queue_size as i32)
        .bind(queue.ring_timeout.as_secs() as i64)
        .bind(queue.retry_delay.as_secs() as i64)
        .bind(queue.max_retries as i32)
        .bind(queue.wrap_up_time.as_secs() as i64)
        .bind(queue.announce_position)
        .bind(queue.announce_wait_time)
        .bind(queue.music_on_hold.as_ref())
        .bind(queue.periodic_announce.as_ref())
        .bind(queue.periodic_announce_frequency.as_secs() as i64)
        .bind(queue.overflow_queue_id)
        .bind(&overflow_action_str)
        .bind(queue.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated call queue: {}", queue.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update call queue: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn delete_queue(&self, queue_id: Uuid) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM call_queues WHERE id = $1")
            .bind(queue_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Deleted call queue: {}", queue_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete call queue: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn list_queues(&self) -> Result<Vec<CallQueue>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, extension, strategy, max_wait_time_secs, max_queue_size, ring_timeout_secs,
                   retry_delay_secs, max_retries, wrap_up_time_secs, announce_position, announce_wait_time,
                   music_on_hold, periodic_announce, periodic_announce_frequency_secs,
                   overflow_queue_id, overflow_action, created_at, updated_at
            FROM call_queues
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let queues: Vec<CallQueue> = rows
                    .iter()
                    .map(|row| {
                        let strategy_str: String = row.get("strategy");
                        let strategy = match strategy_str.as_str() {
                            "RingAll" => QueueStrategy::RingAll,
                            "Linear" => QueueStrategy::Linear,
                            "LeastRecent" => QueueStrategy::LeastRecent,
                            "FewestCalls" => QueueStrategy::FewestCalls,
                            "LeastTalkTime" => QueueStrategy::LeastTalkTime,
                            "Random" => QueueStrategy::Random,
                            "RoundRobin" => QueueStrategy::RoundRobin,
                            _ => QueueStrategy::RoundRobin,
                        };

                        let overflow_action_str: String = row.get("overflow_action");
                        let overflow_action = match overflow_action_str.as_str() {
                            "Busy" => OverflowAction::Busy,
                            "Voicemail" => OverflowAction::Voicemail,
                            "ForwardToQueue" => OverflowAction::ForwardToQueue,
                            "ForwardToExtension" => OverflowAction::ForwardToExtension,
                            "Announcement" => OverflowAction::Announcement,
                            _ => OverflowAction::Busy,
                        };

                        CallQueue {
                            id: row.get("id"),
                            name: row.get("name"),
                            extension: row.get("extension"),
                            strategy,
                            max_wait_time: Duration::from_secs(row.get::<i64, _>("max_wait_time_secs") as u64),
                            max_queue_size: row.get::<i32, _>("max_queue_size") as usize,
                            ring_timeout: Duration::from_secs(row.get::<i64, _>("ring_timeout_secs") as u64),
                            retry_delay: Duration::from_secs(row.get::<i64, _>("retry_delay_secs") as u64),
                            max_retries: row.get::<i32, _>("max_retries") as u32,
                            wrap_up_time: Duration::from_secs(row.get::<i64, _>("wrap_up_time_secs") as u64),
                            announce_position: row.get("announce_position"),
                            announce_wait_time: row.get("announce_wait_time"),
                            music_on_hold: row.get("music_on_hold"),
                            periodic_announce: row.get("periodic_announce"),
                            periodic_announce_frequency: Duration::from_secs(
                                row.get::<i64, _>("periodic_announce_frequency_secs") as u64,
                            ),
                            overflow_queue_id: row.get("overflow_queue_id"),
                            overflow_action,
                            created_at: row.get("created_at"),
                            updated_at: row.get("updated_at"),
                        }
                    })
                    .collect();

                Ok(queues)
            }
            Err(e) => {
                error!("Failed to list call queues: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn add_member(&self, queue_id: Uuid, member: QueueMember) -> Result<(), String> {
        let status_str = format!("{:?}", member.status);

        let result = sqlx::query(
            r#"
            INSERT INTO queue_members
            (id, queue_id, user_id, username, extension, status, penalty, paused, paused_reason,
             last_call_time, total_calls, answered_calls, missed_calls, total_talk_time_secs, joined_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(member.id)
        .bind(queue_id)
        .bind(member.user_id)
        .bind(&member.username)
        .bind(&member.extension)
        .bind(&status_str)
        .bind(member.penalty as i32)
        .bind(member.paused)
        .bind(member.paused_reason.as_ref())
        .bind(member.last_call_time)
        .bind(member.total_calls as i64)
        .bind(member.answered_calls as i64)
        .bind(member.missed_calls as i64)
        .bind(member.total_talk_time.as_secs() as i64)
        .bind(member.joined_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Added member {} to queue {}", member.id, queue_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to add member to queue: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn remove_member(&self, _queue_id: Uuid, member_id: Uuid) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM queue_members WHERE id = $1")
            .bind(member_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Removed member: {}", member_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove member: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_member(&self, member: &QueueMember) -> Result<(), String> {
        let status_str = format!("{:?}", member.status);

        let result = sqlx::query(
            r#"
            UPDATE queue_members
            SET username = $2, extension = $3, status = $4, penalty = $5,
                paused = $6, paused_reason = $7, last_call_time = $8,
                total_calls = $9, answered_calls = $10, missed_calls = $11,
                total_talk_time_secs = $12
            WHERE id = $1
            "#,
        )
        .bind(member.id)
        .bind(&member.username)
        .bind(&member.extension)
        .bind(&status_str)
        .bind(member.penalty as i32)
        .bind(member.paused)
        .bind(member.paused_reason.as_ref())
        .bind(member.last_call_time)
        .bind(member.total_calls as i64)
        .bind(member.answered_calls as i64)
        .bind(member.missed_calls as i64)
        .bind(member.total_talk_time.as_secs() as i64)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated member: {}", member.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update member: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_members(&self, queue_id: Uuid) -> Result<Vec<QueueMember>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, queue_id, user_id, username, extension, status, penalty, paused, paused_reason,
                   last_call_time, total_calls, answered_calls, missed_calls, total_talk_time_secs, joined_at
            FROM queue_members
            WHERE queue_id = $1
            ORDER BY joined_at
            "#,
        )
        .bind(queue_id)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let members: Vec<QueueMember> = rows
                    .iter()
                    .map(|row| {
                        let status_str: String = row.get("status");
                        let status = match status_str.as_str() {
                            "Available" => AgentStatus::Available,
                            "Busy" => AgentStatus::Busy,
                            "AfterCallWork" => AgentStatus::AfterCallWork,
                            "Paused" => AgentStatus::Paused,
                            "LoggedOut" => AgentStatus::LoggedOut,
                            _ => AgentStatus::Available,
                        };

                        QueueMember {
                            id: row.get("id"),
                            user_id: row.get("user_id"),
                            username: row.get("username"),
                            extension: row.get("extension"),
                            status,
                            penalty: row.get::<i32, _>("penalty") as u32,
                            paused: row.get("paused"),
                            paused_reason: row.get("paused_reason"),
                            last_call_time: row.get("last_call_time"),
                            total_calls: row.get::<i64, _>("total_calls") as u64,
                            answered_calls: row.get::<i64, _>("answered_calls") as u64,
                            missed_calls: row.get::<i64, _>("missed_calls") as u64,
                            total_talk_time: Duration::from_secs(
                                row.get::<i64, _>("total_talk_time_secs") as u64,
                            ),
                            joined_at: row.get("joined_at"),
                        }
                    })
                    .collect();

                Ok(members)
            }
            Err(e) => {
                error!("Failed to get queue members: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_member(&self, member_id: Uuid) -> Result<Option<QueueMember>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, queue_id, user_id, username, extension, status, penalty, paused, paused_reason,
                   last_call_time, total_calls, answered_calls, missed_calls, total_talk_time_secs, joined_at
            FROM queue_members
            WHERE id = $1
            "#,
        )
        .bind(member_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "Available" => AgentStatus::Available,
                    "Busy" => AgentStatus::Busy,
                    "AfterCallWork" => AgentStatus::AfterCallWork,
                    "Paused" => AgentStatus::Paused,
                    "LoggedOut" => AgentStatus::LoggedOut,
                    _ => AgentStatus::Available,
                };

                let member = QueueMember {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    username: row.get("username"),
                    extension: row.get("extension"),
                    status,
                    penalty: row.get::<i32, _>("penalty") as u32,
                    paused: row.get("paused"),
                    paused_reason: row.get("paused_reason"),
                    last_call_time: row.get("last_call_time"),
                    total_calls: row.get::<i64, _>("total_calls") as u64,
                    answered_calls: row.get::<i64, _>("answered_calls") as u64,
                    missed_calls: row.get::<i64, _>("missed_calls") as u64,
                    total_talk_time: Duration::from_secs(
                        row.get::<i64, _>("total_talk_time_secs") as u64,
                    ),
                    joined_at: row.get("joined_at"),
                };

                Ok(Some(member))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get member: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::call_queue::{CallQueue, QueueMember, QueueStrategy};

    // Integration tests require a database connection
    // Run with: cargo test --test call_queue_repository_test

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_and_get_queue() {
        // Test implementation would go here
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_add_and_get_members() {
        // Test implementation would go here
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_update_member() {
        // Test implementation would go here
    }
}
