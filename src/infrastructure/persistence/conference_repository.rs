/// PostgreSQL implementation of ConferenceRepository
use crate::domain::conference::{
    ConferenceRepository, ConferenceRoom, ConferenceState, Participant, ParticipantRole,
    ParticipantState,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::{debug, error};
use uuid::Uuid;

pub struct PgConferenceRepository {
    pool: PgPool,
}

impl PgConferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConferenceRepository for PgConferenceRepository {
    async fn create_room(&self, room: ConferenceRoom) -> Result<ConferenceRoom, String> {
        let state_str = format!("{:?}", room.state);

        let result = sqlx::query(
            r#"
            INSERT INTO conference_rooms
            (id, name, pin, max_participants, state, moderator_id, recording_enabled, recording_file, created_at, started_at, ended_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(room.id)
        .bind(&room.name)
        .bind(room.pin.as_ref())
        .bind(room.max_participants as i32)
        .bind(&state_str)
        .bind(room.moderator_id)
        .bind(room.recording_enabled)
        .bind(room.recording_file.as_ref())
        .bind(room.created_at)
        .bind(room.started_at)
        .bind(room.ended_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Created conference room: {}", room.id);
                Ok(room)
            }
            Err(e) => {
                error!("Failed to create conference room: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_room(&self, room_id: Uuid) -> Result<Option<ConferenceRoom>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, pin, max_participants, state, moderator_id,
                   recording_enabled, recording_file, created_at, started_at, ended_at
            FROM conference_rooms
            WHERE id = $1
            "#,
        )
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let state_str: String = row.get("state");
                let state = match state_str.as_str() {
                    "Waiting" => ConferenceState::Waiting,
                    "Active" => ConferenceState::Active,
                    "Locked" => ConferenceState::Locked,
                    "Ended" => ConferenceState::Ended,
                    _ => ConferenceState::Waiting,
                };

                let room = ConferenceRoom {
                    id: row.get("id"),
                    name: row.get("name"),
                    pin: row.get("pin"),
                    max_participants: row.get::<i32, _>("max_participants") as usize,
                    state,
                    participants: HashMap::new(), // Will load separately
                    moderator_id: row.get("moderator_id"),
                    recording_enabled: row.get("recording_enabled"),
                    recording_file: row.get("recording_file"),
                    created_at: row.get("created_at"),
                    started_at: row.get("started_at"),
                    ended_at: row.get("ended_at"),
                };

                Ok(Some(room))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get conference room: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_room(&self, room: &ConferenceRoom) -> Result<(), String> {
        let state_str = format!("{:?}", room.state);

        let result = sqlx::query(
            r#"
            UPDATE conference_rooms
            SET name = $2, pin = $3, max_participants = $4, state = $5,
                moderator_id = $6, recording_enabled = $7, recording_file = $8,
                started_at = $9, ended_at = $10
            WHERE id = $1
            "#,
        )
        .bind(room.id)
        .bind(&room.name)
        .bind(room.pin.as_ref())
        .bind(room.max_participants as i32)
        .bind(&state_str)
        .bind(room.moderator_id)
        .bind(room.recording_enabled)
        .bind(room.recording_file.as_ref())
        .bind(room.started_at)
        .bind(room.ended_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated conference room: {}", room.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update conference room: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn delete_room(&self, room_id: Uuid) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM conference_rooms WHERE id = $1")
            .bind(room_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Deleted conference room: {}", room_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete conference room: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn list_rooms(&self, state: Option<ConferenceState>) -> Result<Vec<ConferenceRoom>, String> {
        let query = if let Some(state) = state {
            let state_str = format!("{:?}", state);
            sqlx::query(
                r#"
                SELECT id, name, pin, max_participants, state, moderator_id,
                       recording_enabled, recording_file, created_at, started_at, ended_at
                FROM conference_rooms
                WHERE state = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(&state_str)
        } else {
            sqlx::query(
                r#"
                SELECT id, name, pin, max_participants, state, moderator_id,
                       recording_enabled, recording_file, created_at, started_at, ended_at
                FROM conference_rooms
                ORDER BY created_at DESC
                "#,
            )
        };

        let result = query.fetch_all(&self.pool).await;

        match result {
            Ok(rows) => {
                let rooms: Vec<ConferenceRoom> = rows
                    .into_iter()
                    .map(|row| {
                        let state_str: String = row.get("state");
                        let state = match state_str.as_str() {
                            "Waiting" => ConferenceState::Waiting,
                            "Active" => ConferenceState::Active,
                            "Locked" => ConferenceState::Locked,
                            "Ended" => ConferenceState::Ended,
                            _ => ConferenceState::Waiting,
                        };

                        ConferenceRoom {
                            id: row.get("id"),
                            name: row.get("name"),
                            pin: row.get("pin"),
                            max_participants: row.get::<i32, _>("max_participants") as usize,
                            state,
                            participants: HashMap::new(),
                            moderator_id: row.get("moderator_id"),
                            recording_enabled: row.get("recording_enabled"),
                            recording_file: row.get("recording_file"),
                            created_at: row.get("created_at"),
                            started_at: row.get("started_at"),
                            ended_at: row.get("ended_at"),
                        }
                    })
                    .collect();

                Ok(rooms)
            }
            Err(e) => {
                error!("Failed to list conference rooms: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn add_participant(&self, room_id: Uuid, participant: Participant) -> Result<(), String> {
        let role_str = format!("{:?}", participant.role);
        let state_str = format!("{:?}", participant.state);

        let result = sqlx::query(
            r#"
            INSERT INTO conference_participants
            (id, room_id, name, call_id, role, state, is_muted, volume, joined_at, left_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(participant.id)
        .bind(room_id)
        .bind(&participant.name)
        .bind(&participant.call_id)
        .bind(&role_str)
        .bind(&state_str)
        .bind(participant.is_muted)
        .bind(participant.volume)
        .bind(participant.joined_at)
        .bind(participant.left_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Added participant {} to room {}", participant.id, room_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to add participant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn remove_participant(&self, room_id: Uuid, participant_id: Uuid) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            UPDATE conference_participants
            SET left_at = NOW()
            WHERE room_id = $1 AND id = $2
            "#,
        )
        .bind(room_id)
        .bind(participant_id)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Removed participant {} from room {}", participant_id, room_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove participant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_participants(&self, room_id: Uuid) -> Result<Vec<Participant>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, call_id, role, state, is_muted, volume, joined_at, left_at
            FROM conference_participants
            WHERE room_id = $1 AND left_at IS NULL
            ORDER BY joined_at ASC
            "#,
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let participants: Vec<Participant> = rows
                    .into_iter()
                    .map(|row| {
                        let role_str: String = row.get("role");
                        let role = match role_str.as_str() {
                            "Moderator" => ParticipantRole::Moderator,
                            "Presenter" => ParticipantRole::Presenter,
                            "Attendee" => ParticipantRole::Attendee,
                            "Listener" => ParticipantRole::Listener,
                            _ => ParticipantRole::Attendee,
                        };

                        let state_str: String = row.get("state");
                        let state = match state_str.as_str() {
                            "Connecting" => ParticipantState::Connecting,
                            "Active" => ParticipantState::Active,
                            "OnHold" => ParticipantState::OnHold,
                            "Muted" => ParticipantState::Muted,
                            "Disconnected" => ParticipantState::Disconnected,
                            _ => ParticipantState::Active,
                        };

                        Participant {
                            id: row.get("id"),
                            name: row.get("name"),
                            call_id: row.get("call_id"),
                            role,
                            state,
                            is_muted: row.get("is_muted"),
                            volume: row.get("volume"),
                            joined_at: row.get("joined_at"),
                            left_at: row.get("left_at"),
                        }
                    })
                    .collect();

                Ok(participants)
            }
            Err(e) => {
                error!("Failed to get participants: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_participant(
        &self,
        room_id: Uuid,
        participant_id: Uuid,
        participant: &Participant,
    ) -> Result<(), String> {
        let role_str = format!("{:?}", participant.role);
        let state_str = format!("{:?}", participant.state);

        let result = sqlx::query(
            r#"
            UPDATE conference_participants
            SET name = $3, role = $4, state = $5, is_muted = $6, volume = $7
            WHERE room_id = $1 AND id = $2
            "#,
        )
        .bind(room_id)
        .bind(participant_id)
        .bind(&participant.name)
        .bind(&role_str)
        .bind(&state_str)
        .bind(participant.is_muted)
        .bind(participant.volume)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated participant {} in room {}", participant_id, room_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update participant: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn find_rooms_by_user(&self, _user_id: i32) -> Result<Vec<ConferenceRoom>, String> {
        // This would require joining with user_id if we track that
        // For now, return all active rooms
        self.list_rooms(Some(ConferenceState::Active)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Note: These tests require a running PostgreSQL database with migrations applied
    // Run with: cargo test --features postgres conference_repository

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://yakyak:password@localhost/yakyak_test".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_and_get_room() {
        let pool = setup_test_pool().await;
        let repo = PgConferenceRepository::new(pool);

        let room = ConferenceRoom::new(
            "Test Room".to_string(),
            Some("1234".to_string()),
            10,
        );
        let room_id = room.id;

        repo.create_room(room).await.unwrap();

        let retrieved = repo.get_room(room_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_room = retrieved.unwrap();
        assert_eq!(retrieved_room.name, "Test Room");
        assert_eq!(retrieved_room.pin, Some("1234".to_string()));
        assert_eq!(retrieved_room.max_participants, 10);

        // Cleanup
        repo.delete_room(room_id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_list_rooms() {
        let pool = setup_test_pool().await;
        let repo = PgConferenceRepository::new(pool);

        let room1 = ConferenceRoom::new("Room 1".to_string(), None, 5);
        let room2 = ConferenceRoom::new("Room 2".to_string(), None, 10);

        repo.create_room(room1.clone()).await.unwrap();
        repo.create_room(room2.clone()).await.unwrap();

        let rooms = repo.list_rooms(None).await.unwrap();
        assert!(rooms.len() >= 2);

        // Cleanup
        repo.delete_room(room1.id).await.unwrap();
        repo.delete_room(room2.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_add_and_get_participants() {
        let pool = setup_test_pool().await;
        let repo = PgConferenceRepository::new(pool);

        let room = ConferenceRoom::new("Test Room".to_string(), None, 10);
        let room_id = room.id;

        repo.create_room(room).await.unwrap();

        let participant = Participant::new(
            "Alice".to_string(),
            "call-123".to_string(),
            ParticipantRole::Moderator,
        );
        let participant_id = participant.id;

        repo.add_participant(room_id, participant).await.unwrap();

        let participants = repo.get_participants(room_id).await.unwrap();
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0].name, "Alice");
        assert_eq!(participants[0].role, ParticipantRole::Moderator);

        // Cleanup
        repo.remove_participant(room_id, participant_id).await.unwrap();
        repo.delete_room(room_id).await.unwrap();
    }
}
