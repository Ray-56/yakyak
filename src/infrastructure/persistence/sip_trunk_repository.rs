/// PostgreSQL implementation of SipTrunkRepository
use crate::domain::sip_trunk::{
    CodecPreference, DtmfMode, SipTrunk, SipTrunkRepository, TrunkDirection, TrunkStatistics,
    TrunkType,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::{debug, error};
use uuid::Uuid;

pub struct PgSipTrunkRepository {
    pool: PgPool,
}

impl PgSipTrunkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SipTrunkRepository for PgSipTrunkRepository {
    async fn create_trunk(&self, trunk: SipTrunk) -> Result<SipTrunk, String> {
        let trunk_type_str = format!("{:?}", trunk.trunk_type);
        let direction_str = format!("{:?}", trunk.direction);
        let dtmf_mode_str = format!("{:?}", trunk.dtmf_mode);
        let allowed_ips_str = trunk.allowed_ips.join(",");
        let codecs_str = trunk
            .codecs
            .iter()
            .map(|c| format!("{}:{}", c.codec, c.priority))
            .collect::<Vec<_>>()
            .join(",");

        let result = sqlx::query(
            r#"
            INSERT INTO sip_trunks
            (id, name, provider_name, trunk_type, sip_server, sip_port, backup_server,
             direction, username, password, auth_username, realm, allowed_ips,
             register_enabled, registration_interval, codecs, dtmf_mode,
             max_concurrent_calls, max_calls_per_second, caller_id_number, caller_id_name,
             prefix_strip, prefix_add, rtcp_enabled, t38_enabled, srtp_enabled,
             enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
            "#,
        )
        .bind(trunk.id)
        .bind(&trunk.name)
        .bind(&trunk.provider_name)
        .bind(&trunk_type_str)
        .bind(&trunk.sip_server)
        .bind(trunk.sip_port as i32)
        .bind(trunk.backup_server.as_ref())
        .bind(&direction_str)
        .bind(trunk.username.as_ref())
        .bind(trunk.password.as_ref())
        .bind(trunk.auth_username.as_ref())
        .bind(trunk.realm.as_ref())
        .bind(&allowed_ips_str)
        .bind(trunk.register_enabled)
        .bind(trunk.registration_interval as i64)
        .bind(&codecs_str)
        .bind(&dtmf_mode_str)
        .bind(trunk.max_concurrent_calls as i32)
        .bind(trunk.max_calls_per_second as i32)
        .bind(trunk.caller_id_number.as_ref())
        .bind(trunk.caller_id_name.as_ref())
        .bind(trunk.prefix_strip.as_ref())
        .bind(trunk.prefix_add.as_ref())
        .bind(trunk.rtcp_enabled)
        .bind(trunk.t38_enabled)
        .bind(trunk.srtp_enabled)
        .bind(trunk.enabled)
        .bind(trunk.created_at)
        .bind(trunk.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Created SIP trunk: {}", trunk.id);
                Ok(trunk)
            }
            Err(e) => {
                error!("Failed to create SIP trunk: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_trunk(&self, trunk_id: Uuid) -> Result<Option<SipTrunk>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, provider_name, trunk_type, sip_server, sip_port, backup_server,
                   direction, username, password, auth_username, realm, allowed_ips,
                   register_enabled, registration_interval, registration_expires_at, registered,
                   last_registration_time, codecs, dtmf_mode, max_concurrent_calls, max_calls_per_second,
                   caller_id_number, caller_id_name, prefix_strip, prefix_add, rtcp_enabled, t38_enabled,
                   srtp_enabled, enabled, created_at, updated_at
            FROM sip_trunks
            WHERE id = $1
            "#,
        )
        .bind(trunk_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(row_to_trunk(row))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get SIP trunk: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_trunk_by_name(&self, name: &str) -> Result<Option<SipTrunk>, String> {
        let result = sqlx::query(
            r#"
            SELECT id, name, provider_name, trunk_type, sip_server, sip_port, backup_server,
                   direction, username, password, auth_username, realm, allowed_ips,
                   register_enabled, registration_interval, registration_expires_at, registered,
                   last_registration_time, codecs, dtmf_mode, max_concurrent_calls, max_calls_per_second,
                   caller_id_number, caller_id_name, prefix_strip, prefix_add, rtcp_enabled, t38_enabled,
                   srtp_enabled, enabled, created_at, updated_at
            FROM sip_trunks
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(row_to_trunk(row))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get SIP trunk by name: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_trunk(&self, trunk: &SipTrunk) -> Result<(), String> {
        let trunk_type_str = format!("{:?}", trunk.trunk_type);
        let direction_str = format!("{:?}", trunk.direction);
        let dtmf_mode_str = format!("{:?}", trunk.dtmf_mode);
        let allowed_ips_str = trunk.allowed_ips.join(",");
        let codecs_str = trunk
            .codecs
            .iter()
            .map(|c| format!("{}:{}", c.codec, c.priority))
            .collect::<Vec<_>>()
            .join(",");

        let result = sqlx::query(
            r#"
            UPDATE sip_trunks
            SET name = $2, provider_name = $3, trunk_type = $4, sip_server = $5, sip_port = $6,
                backup_server = $7, direction = $8, username = $9, password = $10, auth_username = $11,
                realm = $12, allowed_ips = $13, register_enabled = $14, registration_interval = $15,
                registration_expires_at = $16, registered = $17, last_registration_time = $18,
                codecs = $19, dtmf_mode = $20, max_concurrent_calls = $21, max_calls_per_second = $22,
                caller_id_number = $23, caller_id_name = $24, prefix_strip = $25, prefix_add = $26,
                rtcp_enabled = $27, t38_enabled = $28, srtp_enabled = $29, enabled = $30, updated_at = $31
            WHERE id = $1
            "#,
        )
        .bind(trunk.id)
        .bind(&trunk.name)
        .bind(&trunk.provider_name)
        .bind(&trunk_type_str)
        .bind(&trunk.sip_server)
        .bind(trunk.sip_port as i32)
        .bind(trunk.backup_server.as_ref())
        .bind(&direction_str)
        .bind(trunk.username.as_ref())
        .bind(trunk.password.as_ref())
        .bind(trunk.auth_username.as_ref())
        .bind(trunk.realm.as_ref())
        .bind(&allowed_ips_str)
        .bind(trunk.register_enabled)
        .bind(trunk.registration_interval as i64)
        .bind(trunk.registration_expires_at)
        .bind(trunk.registered)
        .bind(trunk.last_registration_time)
        .bind(&codecs_str)
        .bind(&dtmf_mode_str)
        .bind(trunk.max_concurrent_calls as i32)
        .bind(trunk.max_calls_per_second as i32)
        .bind(trunk.caller_id_number.as_ref())
        .bind(trunk.caller_id_name.as_ref())
        .bind(trunk.prefix_strip.as_ref())
        .bind(trunk.prefix_add.as_ref())
        .bind(trunk.rtcp_enabled)
        .bind(trunk.t38_enabled)
        .bind(trunk.srtp_enabled)
        .bind(trunk.enabled)
        .bind(trunk.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated SIP trunk: {}", trunk.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update SIP trunk: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn delete_trunk(&self, trunk_id: Uuid) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM sip_trunks WHERE id = $1")
            .bind(trunk_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Deleted SIP trunk: {}", trunk_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete SIP trunk: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn list_trunks(&self, enabled_only: bool) -> Result<Vec<SipTrunk>, String> {
        let result = if enabled_only {
            sqlx::query(
                r#"
                SELECT id, name, provider_name, trunk_type, sip_server, sip_port, backup_server,
                       direction, username, password, auth_username, realm, allowed_ips,
                       register_enabled, registration_interval, registration_expires_at, registered,
                       last_registration_time, codecs, dtmf_mode, max_concurrent_calls, max_calls_per_second,
                       caller_id_number, caller_id_name, prefix_strip, prefix_add, rtcp_enabled, t38_enabled,
                       srtp_enabled, enabled, created_at, updated_at
                FROM sip_trunks
                WHERE enabled = TRUE
                ORDER BY name
                "#,
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, name, provider_name, trunk_type, sip_server, sip_port, backup_server,
                       direction, username, password, auth_username, realm, allowed_ips,
                       register_enabled, registration_interval, registration_expires_at, registered,
                       last_registration_time, codecs, dtmf_mode, max_concurrent_calls, max_calls_per_second,
                       caller_id_number, caller_id_name, prefix_strip, prefix_add, rtcp_enabled, t38_enabled,
                       srtp_enabled, enabled, created_at, updated_at
                FROM sip_trunks
                ORDER BY name
                "#,
            )
            .fetch_all(&self.pool)
            .await
        };

        match result {
            Ok(rows) => {
                let trunks: Vec<SipTrunk> = rows.iter().map(|row| row_to_trunk(row.clone())).collect();
                Ok(trunks)
            }
            Err(e) => {
                error!("Failed to list SIP trunks: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn get_statistics(&self, trunk_id: Uuid) -> Result<Option<TrunkStatistics>, String> {
        let result = sqlx::query(
            r#"
            SELECT trunk_id, current_calls, total_calls, successful_calls, failed_calls,
                   average_call_duration, total_minutes, last_call_time
            FROM trunk_statistics
            WHERE trunk_id = $1
            "#,
        )
        .bind(trunk_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let stats = TrunkStatistics {
                    trunk_id: row.get("trunk_id"),
                    current_calls: row.get::<i32, _>("current_calls") as u32,
                    total_calls: row.get::<i64, _>("total_calls") as u64,
                    successful_calls: row.get::<i64, _>("successful_calls") as u64,
                    failed_calls: row.get::<i64, _>("failed_calls") as u64,
                    average_call_duration: row.get("average_call_duration"),
                    total_minutes: row.get("total_minutes"),
                    last_call_time: row.get("last_call_time"),
                };
                Ok(Some(stats))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get trunk statistics: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    async fn update_statistics(&self, stats: &TrunkStatistics) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            INSERT INTO trunk_statistics
            (trunk_id, current_calls, total_calls, successful_calls, failed_calls,
             average_call_duration, total_minutes, last_call_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (trunk_id)
            DO UPDATE SET
                current_calls = $2,
                total_calls = $3,
                successful_calls = $4,
                failed_calls = $5,
                average_call_duration = $6,
                total_minutes = $7,
                last_call_time = $8
            "#,
        )
        .bind(stats.trunk_id)
        .bind(stats.current_calls as i32)
        .bind(stats.total_calls as i64)
        .bind(stats.successful_calls as i64)
        .bind(stats.failed_calls as i64)
        .bind(stats.average_call_duration)
        .bind(stats.total_minutes)
        .bind(stats.last_call_time)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Updated statistics for trunk: {}", stats.trunk_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update trunk statistics: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }
}

fn row_to_trunk(row: sqlx::postgres::PgRow) -> SipTrunk {
    let trunk_type_str: String = row.get("trunk_type");
    let trunk_type = match trunk_type_str.as_str() {
        "Register" => TrunkType::Register,
        "IpBased" => TrunkType::IpBased,
        "Peer" => TrunkType::Peer,
        _ => TrunkType::Register,
    };

    let direction_str: String = row.get("direction");
    let direction = match direction_str.as_str() {
        "Inbound" => TrunkDirection::Inbound,
        "Outbound" => TrunkDirection::Outbound,
        "Bidirectional" => TrunkDirection::Bidirectional,
        _ => TrunkDirection::Bidirectional,
    };

    let dtmf_mode_str: String = row.get("dtmf_mode");
    let dtmf_mode = match dtmf_mode_str.as_str() {
        "Rfc2833" => DtmfMode::Rfc2833,
        "SipInfo" => DtmfMode::SipInfo,
        "Inband" => DtmfMode::Inband,
        _ => DtmfMode::Rfc2833,
    };

    let allowed_ips_str: String = row.get("allowed_ips");
    let allowed_ips: Vec<String> = allowed_ips_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let codecs_str: String = row.get("codecs");
    let codecs: Vec<CodecPreference> = codecs_str
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 2 {
                let codec = parts[0].to_string();
                let priority: u8 = parts[1].parse().ok()?;
                Some(CodecPreference { codec, priority })
            } else {
                None
            }
        })
        .collect();

    SipTrunk {
        id: row.get("id"),
        name: row.get("name"),
        provider_name: row.get("provider_name"),
        trunk_type,
        sip_server: row.get("sip_server"),
        sip_port: row.get::<i32, _>("sip_port") as u16,
        backup_server: row.get("backup_server"),
        direction,
        username: row.get("username"),
        password: row.get("password"),
        auth_username: row.get("auth_username"),
        realm: row.get("realm"),
        allowed_ips,
        register_enabled: row.get("register_enabled"),
        registration_interval: row.get::<i64, _>("registration_interval") as u64,
        registration_expires_at: row.get("registration_expires_at"),
        registered: row.get("registered"),
        last_registration_time: row.get("last_registration_time"),
        codecs,
        dtmf_mode,
        max_concurrent_calls: row.get::<i32, _>("max_concurrent_calls") as u32,
        max_calls_per_second: row.get::<i32, _>("max_calls_per_second") as u32,
        caller_id_number: row.get("caller_id_number"),
        caller_id_name: row.get("caller_id_name"),
        prefix_strip: row.get("prefix_strip"),
        prefix_add: row.get("prefix_add"),
        rtcp_enabled: row.get("rtcp_enabled"),
        t38_enabled: row.get("t38_enabled"),
        srtp_enabled: row.get("srtp_enabled"),
        enabled: row.get("enabled"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_and_get_trunk() {
        // Test implementation would go here
    }
}
