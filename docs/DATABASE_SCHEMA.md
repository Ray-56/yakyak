# Database Schema Documentation

YakYak uses PostgreSQL as its primary database for storing users, roles, call detail records (CDR), and other system data.

## Tables

### users

Stores user account information for SIP authentication and system access.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| id | SERIAL | NO | Primary key |
| username | VARCHAR(255) | NO | Unique username (also used as SIP username) |
| password_hash | VARCHAR(255) | NO | bcrypt hashed password |
| sip_ha1 | VARCHAR(255) | YES | MD5(username:realm:password) for SIP Digest auth |
| realm | VARCHAR(255) | NO | SIP realm (typically domain name) |
| display_name | VARCHAR(255) | YES | User's display name |
| email | VARCHAR(255) | YES | User's email address |
| enabled | BOOLEAN | NO | Whether user account is active (default: true) |
| role_id | UUID | YES | Foreign key to roles table |
| created_at | TIMESTAMP | NO | Account creation timestamp |
| updated_at | TIMESTAMP | NO | Last update timestamp |

**Indexes:**
- PRIMARY KEY: `id`
- UNIQUE: `username`
- INDEX: `realm`
- INDEX: `enabled`
- INDEX: `role_id`

**Triggers:**
- `update_users_updated_at_trigger`: Automatically updates `updated_at` on row update

### roles

Stores user roles for role-based access control (RBAC).

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| id | UUID | NO | Primary key |
| name | VARCHAR(255) | NO | Unique role name |
| description | TEXT | YES | Role description |
| permissions | TEXT[] | NO | Array of permission strings (e.g., "user:read", "call:create") |
| is_system | BOOLEAN | NO | System roles cannot be deleted (default: false) |
| created_at | TIMESTAMP | NO | Role creation timestamp |
| updated_at | TIMESTAMP | NO | Last update timestamp |

**Indexes:**
- PRIMARY KEY: `id`
- UNIQUE: `name`
- INDEX: `is_system`

**Default Roles:**
1. **administrator**: Full system access with all permissions
2. **user**: Standard user with basic call permissions
3. **operator**: Call center operator with call management permissions

**Triggers:**
- `update_roles_updated_at_trigger`: Automatically updates `updated_at` on row update

### call_records (CDR)

Stores call detail records for billing, analytics, and auditing.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| id | UUID | NO | Primary key |
| call_id | VARCHAR(255) | NO | SIP Call-ID header value |
| caller | VARCHAR(255) | NO | Caller SIP URI |
| callee | VARCHAR(255) | NO | Callee SIP URI |
| caller_ip | VARCHAR(45) | YES | Caller IP address |
| callee_ip | VARCHAR(45) | YES | Callee IP address |
| direction | VARCHAR(50) | NO | Call direction (Inbound/Outbound/Internal) |
| start_time | TIMESTAMP | NO | Call start time |
| answer_time | TIMESTAMP | YES | Time call was answered |
| end_time | TIMESTAMP | YES | Call end time |
| setup_duration_ms | INTEGER | YES | Time from INVITE to answer (milliseconds) |
| call_duration_ms | INTEGER | YES | Time from answer to hangup (milliseconds) |
| total_duration_ms | INTEGER | YES | Total call time (milliseconds) |
| status | VARCHAR(50) | NO | Call status (Active/Completed/Failed/Busy/NoAnswer/Cancelled/Rejected) |
| disconnect_reason | TEXT | YES | Reason for call termination |
| codec | VARCHAR(50) | YES | Audio codec used (e.g., PCMU, PCMA) |
| caller_sdp | TEXT | YES | Caller's SDP offer |
| callee_sdp | TEXT | YES | Callee's SDP answer |
| caller_user_agent | TEXT | YES | Caller's User-Agent header |
| callee_user_agent | TEXT | YES | Callee's User-Agent header |
| media_info | TEXT | YES | Additional media information (JSON) |
| sip_response_code | INTEGER | YES | Final SIP response code |
| created_at | TIMESTAMP | NO | Record creation timestamp |
| updated_at | TIMESTAMP | NO | Last update timestamp |

**Indexes:**
- PRIMARY KEY: `id`
- UNIQUE: `call_id`
- INDEX: `caller`
- INDEX: `callee`
- INDEX: `direction`
- INDEX: `status`
- INDEX: `start_time`
- INDEX: `answer_time`
- INDEX: `end_time`
- INDEX: `created_at`

**Triggers:**
- `update_call_records_updated_at_trigger`: Automatically updates `updated_at` on row update

## Relationships

```
users N:1 roles
  - users.role_id → roles.id (ON DELETE SET NULL)
```

## Permissions

YakYak uses string-based permissions in the format `resource:action`. Current permissions include:

### User Management
- `user:read` - View user information
- `user:create` - Create new users
- `user:update` - Update user information
- `user:delete` - Delete users
- `user:manage_roles` - Assign roles to users

### Call Management
- `call:read` - View call information
- `call:create` - Initiate calls
- `call:terminate` - Terminate active calls
- `call:transfer` - Transfer calls

### CDR Access
- `cdr:read` - View call detail records
- `cdr:export` - Export CDR data
- `cdr:delete` - Delete CDR records

### System Administration
- `system:config` - Modify system configuration
- `system:monitor` - Access monitoring and metrics
- `system:audit` - View audit logs

### Conference Management
- `conference:create` - Create conference rooms
- `conference:manage` - Manage conference settings
- `conference:moderate` - Moderate conferences

### Voicemail
- `voicemail:access` - Access voicemail
- `voicemail:manage` - Manage voicemail settings

## Migration Files

Migration files are located in the `migrations/` directory and are executed in alphanumeric order:

1. `20251104_01_create_users_table.sql` - Initial users table
2. `202511040002_add_sip_ha1.sql` - Add SIP Digest auth support
3. `202511050003_create_call_records_table.sql` - CDR system
4. `20251106_01_create_roles_table.sql` - Roles and permissions

## Connection Configuration

Database connection is configured via environment variables or `config.toml`:

```toml
[database]
url = "postgres://username:password@localhost/yakyak"
max_connections = 10
min_connections = 2
connect_timeout = 30
idle_timeout = 600
max_lifetime = 1800
```

## Performance Considerations

### Indexes

All foreign keys and frequently queried columns have indexes for optimal query performance.

### Connection Pooling

YakYak uses SQLx connection pooling to manage database connections efficiently:
- Minimum connections: 2
- Maximum connections: 10
- Connection lifetime: 30 minutes

### Query Optimization

- Use prepared statements (automatically done by SQLx)
- Batch inserts when possible
- Use transactions for multi-step operations
- CDR queries are optimized with composite indexes

## Backup and Maintenance

### Recommended Backup Strategy

```bash
# Daily full backup
pg_dump -U yakyak -Fc yakyak > yakyak_$(date +%Y%m%d).dump

# Restore from backup
pg_restore -U yakyak -d yakyak yakyak_20251106.dump
```

### Vacuum and Analyze

Run regularly to maintain performance:

```sql
-- Analyze all tables
ANALYZE;

-- Vacuum (reclaim storage)
VACUUM ANALYZE;
```

### CDR Cleanup

Old CDR records should be periodically archived or deleted:

```sql
-- Delete CDRs older than 1 year
DELETE FROM call_records
WHERE created_at < NOW() - INTERVAL '1 year';
```

Or use the repository method:

```rust
cdr_repository.delete_older_than(365).await?;
```

## Security Considerations

### Password Hashing

- User passwords are hashed with bcrypt (cost factor 12)
- SIP HA1 values are stored for SIP Digest authentication
- Never store plain-text passwords

### Access Control

- Use role-based permissions for all sensitive operations
- Validate user permissions before allowing actions
- Audit sensitive operations in logs

### SQL Injection Prevention

- All queries use parameterized statements (SQLx)
- Never concatenate user input into SQL queries
- Validate and sanitize all user input

## Schema Evolution

When adding new tables or columns:

1. Create a new migration file with timestamp prefix
2. Include both schema changes and data migrations
3. Add rollback statements if possible
4. Test migrations on a copy of production data
5. Document changes in this file

## See Also

- [API Documentation](API.md)
- [Authentication Documentation](../AUTH.md)
- [Call Flow Documentation](../CALL_FLOW.md)
