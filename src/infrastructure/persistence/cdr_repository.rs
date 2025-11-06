//! PostgreSQL implementation of CDR Repository

use crate::domain::cdr::{CallDetailRecord, CallDirection, CallStatus, CdrFilters, CdrRepository};
use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use tracing::{debug, error};
use uuid::Uuid;

#[derive(FromRow)]
struct CdrRow {
    id: Uuid,
    call_id: String,
    caller_username: String,
    caller_uri: String,
    caller_ip: String,
    callee_username: String,
    callee_uri: String,
    callee_ip: Option<String>,
    direction: String,
    start_time: chrono::DateTime<chrono::Utc>,
    answer_time: Option<chrono::DateTime<chrono::Utc>>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    setup_duration: Option<i32>,
    call_duration: Option<i32>,
    total_duration: Option<i32>,
    status: String,
    end_reason: Option<String>,
    sip_response_code: Option<i16>,
    codec: Option<String>,
    rtp_packets_sent: Option<i64>,
    rtp_packets_received: Option<i64>,
    rtp_bytes_sent: Option<i64>,
    rtp_bytes_received: Option<i64>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CdrRow> for CallDetailRecord {
    fn from(r: CdrRow) -> Self {
        CallDetailRecord {
            id: r.id,
            call_id: r.call_id,
            caller_username: r.caller_username,
            caller_uri: r.caller_uri,
            caller_ip: r.caller_ip,
            callee_username: r.callee_username,
            callee_uri: r.callee_uri,
            callee_ip: r.callee_ip,
            direction: match r.direction.as_str() {
                "inbound" => CallDirection::Inbound,
                "outbound" => CallDirection::Outbound,
                _ => CallDirection::Internal,
            },
            start_time: r.start_time,
            answer_time: r.answer_time,
            end_time: r.end_time,
            setup_duration: r.setup_duration,
            call_duration: r.call_duration,
            total_duration: r.total_duration,
            status: CallStatus::from_str(&r.status).unwrap_or(CallStatus::Failed),
            end_reason: r.end_reason,
            sip_response_code: r.sip_response_code.map(|code| code as u16),
            codec: r.codec,
            rtp_packets_sent: r.rtp_packets_sent,
            rtp_packets_received: r.rtp_packets_received,
            rtp_bytes_sent: r.rtp_bytes_sent,
            rtp_bytes_received: r.rtp_bytes_received,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

pub struct PgCdrRepository {
    pool: PgPool,
}

impl PgCdrRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CdrRepository for PgCdrRepository {
    async fn create(&self, cdr: &CallDetailRecord) -> Result<(), String> {
        debug!("Creating CDR for call_id: {}", cdr.call_id);

        sqlx::query!(
            r#"
            INSERT INTO call_records (
                id, call_id,
                caller_username, caller_uri, caller_ip,
                callee_username, callee_uri, callee_ip,
                direction,
                start_time, answer_time, end_time,
                setup_duration, call_duration, total_duration,
                status, end_reason, sip_response_code,
                codec, rtp_packets_sent, rtp_packets_received,
                rtp_bytes_sent, rtp_bytes_received,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            "#,
            cdr.id,
            cdr.call_id,
            cdr.caller_username,
            cdr.caller_uri,
            cdr.caller_ip,
            cdr.callee_username,
            cdr.callee_uri,
            cdr.callee_ip,
            cdr.direction.as_str(),
            cdr.start_time,
            cdr.answer_time,
            cdr.end_time,
            cdr.setup_duration,
            cdr.call_duration,
            cdr.total_duration,
            cdr.status.as_str(),
            cdr.end_reason,
            cdr.sip_response_code.map(|code| code as i16),
            cdr.codec,
            cdr.rtp_packets_sent,
            cdr.rtp_packets_received,
            cdr.rtp_bytes_sent,
            cdr.rtp_bytes_received,
            cdr.created_at,
            cdr.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create CDR: {}", e);
            format!("Database error: {}", e)
        })?;

        debug!("CDR created successfully: {}", cdr.id);
        Ok(())
    }

    async fn update(&self, cdr: &CallDetailRecord) -> Result<(), String> {
        debug!("Updating CDR: {}", cdr.id);

        let result = sqlx::query!(
            r#"
            UPDATE call_records
            SET call_id = $2,
                caller_username = $3, caller_uri = $4, caller_ip = $5,
                callee_username = $6, callee_uri = $7, callee_ip = $8,
                direction = $9,
                start_time = $10, answer_time = $11, end_time = $12,
                setup_duration = $13, call_duration = $14, total_duration = $15,
                status = $16, end_reason = $17, sip_response_code = $18,
                codec = $19, rtp_packets_sent = $20, rtp_packets_received = $21,
                rtp_bytes_sent = $22, rtp_bytes_received = $23,
                updated_at = $24
            WHERE id = $1
            "#,
            cdr.id,
            cdr.call_id,
            cdr.caller_username,
            cdr.caller_uri,
            cdr.caller_ip,
            cdr.callee_username,
            cdr.callee_uri,
            cdr.callee_ip,
            cdr.direction.as_str(),
            cdr.start_time,
            cdr.answer_time,
            cdr.end_time,
            cdr.setup_duration,
            cdr.call_duration,
            cdr.total_duration,
            cdr.status.as_str(),
            cdr.end_reason,
            cdr.sip_response_code.map(|code| code as i16),
            cdr.codec,
            cdr.rtp_packets_sent,
            cdr.rtp_packets_received,
            cdr.rtp_bytes_sent,
            cdr.rtp_bytes_received,
            cdr.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to update CDR: {}", e);
            format!("Database error: {}", e)
        })?;

        if result.rows_affected() == 0 {
            return Err(format!("CDR not found: {}", cdr.id));
        }

        debug!("CDR updated successfully: {}", cdr.id);
        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<CallDetailRecord>, String> {
        debug!("Getting CDR by id: {}", id);

        let record = sqlx::query!(
            r#"
            SELECT
                id, call_id,
                caller_username, caller_uri, caller_ip,
                callee_username, callee_uri, callee_ip,
                direction,
                start_time, answer_time, end_time,
                setup_duration, call_duration, total_duration,
                status, end_reason, sip_response_code,
                codec, rtp_packets_sent, rtp_packets_received,
                rtp_bytes_sent, rtp_bytes_received,
                created_at, updated_at
            FROM call_records
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get CDR: {}", e);
            format!("Database error: {}", e)
        })?;

        Ok(record.map(|r| CallDetailRecord {
            id: r.id,
            call_id: r.call_id,
            caller_username: r.caller_username,
            caller_uri: r.caller_uri,
            caller_ip: r.caller_ip,
            callee_username: r.callee_username,
            callee_uri: r.callee_uri,
            callee_ip: r.callee_ip,
            direction: match r.direction.as_str() {
                "inbound" => CallDirection::Inbound,
                "outbound" => CallDirection::Outbound,
                _ => CallDirection::Internal,
            },
            start_time: r.start_time,
            answer_time: r.answer_time,
            end_time: r.end_time,
            setup_duration: r.setup_duration,
            call_duration: r.call_duration,
            total_duration: r.total_duration,
            status: CallStatus::from_str(&r.status).unwrap_or(CallStatus::Failed),
            end_reason: r.end_reason,
            sip_response_code: r.sip_response_code.map(|code| code as u16),
            codec: r.codec,
            rtp_packets_sent: r.rtp_packets_sent,
            rtp_packets_received: r.rtp_packets_received,
            rtp_bytes_sent: r.rtp_bytes_sent,
            rtp_bytes_received: r.rtp_bytes_received,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn get_by_call_id(&self, call_id: &str) -> Result<Option<CallDetailRecord>, String> {
        debug!("Getting CDR by call_id: {}", call_id);

        let record = sqlx::query!(
            r#"
            SELECT
                id, call_id,
                caller_username, caller_uri, caller_ip,
                callee_username, callee_uri, callee_ip,
                direction,
                start_time, answer_time, end_time,
                setup_duration, call_duration, total_duration,
                status, end_reason, sip_response_code,
                codec, rtp_packets_sent, rtp_packets_received,
                rtp_bytes_sent, rtp_bytes_received,
                created_at, updated_at
            FROM call_records
            WHERE call_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            call_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get CDR by call_id: {}", e);
            format!("Database error: {}", e)
        })?;

        Ok(record.map(|r| CallDetailRecord {
            id: r.id,
            call_id: r.call_id,
            caller_username: r.caller_username,
            caller_uri: r.caller_uri,
            caller_ip: r.caller_ip,
            callee_username: r.callee_username,
            callee_uri: r.callee_uri,
            callee_ip: r.callee_ip,
            direction: match r.direction.as_str() {
                "inbound" => CallDirection::Inbound,
                "outbound" => CallDirection::Outbound,
                _ => CallDirection::Internal,
            },
            start_time: r.start_time,
            answer_time: r.answer_time,
            end_time: r.end_time,
            setup_duration: r.setup_duration,
            call_duration: r.call_duration,
            total_duration: r.total_duration,
            status: CallStatus::from_str(&r.status).unwrap_or(CallStatus::Failed),
            end_reason: r.end_reason,
            sip_response_code: r.sip_response_code.map(|code| code as u16),
            codec: r.codec,
            rtp_packets_sent: r.rtp_packets_sent,
            rtp_packets_received: r.rtp_packets_received,
            rtp_bytes_sent: r.rtp_bytes_sent,
            rtp_bytes_received: r.rtp_bytes_received,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list(
        &self,
        filters: CdrFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CallDetailRecord>, String> {
        debug!("Listing CDRs with filters: {:?}", filters);

        // Use query_as with CdrRow to avoid type mismatch issues
        let records: Vec<CdrRow> = if filters.caller_username.is_none()
            && filters.callee_username.is_none()
            && filters.direction.is_none()
            && filters.status.is_none()
            && filters.start_time_from.is_none()
            && filters.start_time_to.is_none()
            && filters.min_duration.is_none()
        {
            // No filters - simple query
            sqlx::query_as::<_, CdrRow>(
                r#"
                SELECT
                    id, call_id,
                    caller_username, caller_uri, caller_ip,
                    callee_username, callee_uri, callee_ip,
                    direction,
                    start_time, answer_time, end_time,
                    setup_duration, call_duration, total_duration,
                    status, end_reason, sip_response_code,
                    codec, rtp_packets_sent, rtp_packets_received,
                    rtp_bytes_sent, rtp_bytes_received,
                    created_at, updated_at
                FROM call_records
                ORDER BY start_time DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else if let Some(ref caller) = filters.caller_username {
            // With caller filter
            sqlx::query_as::<_, CdrRow>(
                r#"
                SELECT
                    id, call_id,
                    caller_username, caller_uri, caller_ip,
                    callee_username, callee_uri, callee_ip,
                    direction,
                    start_time, answer_time, end_time,
                    setup_duration, call_duration, total_duration,
                    status, end_reason, sip_response_code,
                    codec, rtp_packets_sent, rtp_packets_received,
                    rtp_bytes_sent, rtp_bytes_received,
                    created_at, updated_at
                FROM call_records
                WHERE caller_username = $1
                ORDER BY start_time DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(caller)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            // For other filters, use the no-filter query for now
            sqlx::query_as::<_, CdrRow>(
                r#"
                SELECT
                    id, call_id,
                    caller_username, caller_uri, caller_ip,
                    callee_username, callee_uri, callee_ip,
                    direction,
                    start_time, answer_time, end_time,
                    setup_duration, call_duration, total_duration,
                    status, end_reason, sip_response_code,
                    codec, rtp_packets_sent, rtp_packets_received,
                    rtp_bytes_sent, rtp_bytes_received,
                    created_at, updated_at
                FROM call_records
                ORDER BY start_time DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| {
            error!("Failed to list CDRs: {}", e);
            format!("Database error: {}", e)
        })?;

        Ok(records.into_iter().map(Into::into).collect())
    }

    async fn count(&self, filters: CdrFilters) -> Result<i64, String> {
        debug!("Counting CDRs with filters: {:?}", filters);

        // For simplicity, use basic queries
        let count = if filters.caller_username.is_none()
            && filters.callee_username.is_none()
            && filters.direction.is_none()
            && filters.status.is_none()
            && filters.start_time_from.is_none()
            && filters.start_time_to.is_none()
            && filters.min_duration.is_none()
        {
            sqlx::query_scalar!("SELECT COUNT(*) FROM call_records")
                .fetch_one(&self.pool)
                .await
        } else if let Some(ref caller) = filters.caller_username {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM call_records WHERE caller_username = $1",
                caller
            )
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar!("SELECT COUNT(*) FROM call_records")
                .fetch_one(&self.pool)
                .await
        }
        .map_err(|e| {
            error!("Failed to count CDRs: {}", e);
            format!("Database error: {}", e)
        })?;

        Ok(count.unwrap_or(0))
    }

    async fn delete_older_than(&self, days: i32) -> Result<i64, String> {
        debug!("Deleting CDRs older than {} days", days);

        let result = sqlx::query!(
            r#"
            DELETE FROM call_records
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            "#,
            days as f64
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to delete old CDRs: {}", e);
            format!("Database error: {}", e)
        })?;

        debug!("Deleted {} old CDRs", result.rows_affected());
        Ok(result.rows_affected() as i64)
    }
}
