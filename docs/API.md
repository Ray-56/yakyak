# YakYak REST API Documentation

YakYak provides a RESTful API for managing users, viewing active calls, accessing CDR data, and monitoring system health.

## Base URL

```
http://localhost:8080
```

## Authentication

**Note**: Current version does not require authentication for API endpoints. This will be added in a future release.

## API Endpoints

### Health Check

Check API server health status.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2025-11-06T12:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Service is healthy

---

### User Management

#### Create User

Create a new user account.

**Endpoint:** `POST /users`

**Request Body:**
```json
{
  "username": "alice",
  "password": "secret123",
  "realm": "example.com",
  "display_name": "Alice Smith",
  "email": "alice@example.com",
  "role_id": "a0000000-0000-0000-0000-000000000002"
}
```

**Response:**
```json
{
  "id": 1,
  "username": "alice",
  "realm": "example.com",
  "display_name": "Alice Smith",
  "email": "alice@example.com",
  "enabled": true,
  "role_id": "a0000000-0000-0000-0000-000000000002",
  "created_at": "2025-11-06T12:00:00Z",
  "updated_at": "2025-11-06T12:00:00Z"
}
```

**Status Codes:**
- `201 Created` - User created successfully
- `400 Bad Request` - Invalid request body
- `409 Conflict` - Username already exists

#### Get User

Retrieve user by ID.

**Endpoint:** `GET /users/:id`

**Response:**
```json
{
  "id": 1,
  "username": "alice",
  "realm": "example.com",
  "display_name": "Alice Smith",
  "email": "alice@example.com",
  "enabled": true,
  "role_id": "a0000000-0000-0000-0000-000000000002",
  "created_at": "2025-11-06T12:00:00Z",
  "updated_at": "2025-11-06T12:00:00Z"
}
```

**Status Codes:**
- `200 OK` - User found
- `404 Not Found` - User does not exist

#### Get User by Username

Retrieve user by username.

**Endpoint:** `GET /users/username/:username`

**Response:** Same as Get User

**Status Codes:**
- `200 OK` - User found
- `404 Not Found` - User does not exist

#### List Users

List all users with optional filtering.

**Endpoint:** `GET /users`

**Query Parameters:**
- `realm` (optional) - Filter by realm
- `enabled` (optional) - Filter by enabled status (true/false)

**Response:**
```json
{
  "users": [
    {
      "id": 1,
      "username": "alice",
      "realm": "example.com",
      "display_name": "Alice Smith",
      "email": "alice@example.com",
      "enabled": true,
      "role_id": "a0000000-0000-0000-0000-000000000002",
      "created_at": "2025-11-06T12:00:00Z",
      "updated_at": "2025-11-06T12:00:00Z"
    }
  ],
  "total": 1
}
```

**Status Codes:**
- `200 OK` - List returned successfully

#### Update User

Update user information.

**Endpoint:** `PUT /users/:id`

**Request Body:**
```json
{
  "display_name": "Alice Johnson",
  "email": "alice.j@example.com",
  "enabled": true,
  "role_id": "a0000000-0000-0000-0000-000000000003"
}
```

**Response:** Updated user object

**Status Codes:**
- `200 OK` - User updated successfully
- `404 Not Found` - User does not exist

#### Delete User

Delete a user account.

**Endpoint:** `DELETE /users/:id`

**Response:**
```json
{
  "message": "User deleted successfully"
}
```

**Status Codes:**
- `200 OK` - User deleted successfully
- `404 Not Found` - User does not exist

#### Change Password

Change user password.

**Endpoint:** `POST /users/:id/password`

**Request Body:**
```json
{
  "old_password": "secret123",
  "new_password": "newsecret456"
}
```

**Response:**
```json
{
  "message": "Password changed successfully"
}
```

**Status Codes:**
- `200 OK` - Password changed successfully
- `400 Bad Request` - Invalid old password
- `404 Not Found` - User does not exist

#### Enable/Disable User

Enable or disable a user account.

**Endpoint:** `POST /users/:id/enable` or `POST /users/:id/disable`

**Response:**
```json
{
  "message": "User enabled successfully"
}
```

**Status Codes:**
- `200 OK` - User status changed successfully
- `404 Not Found` - User does not exist

#### Bulk Import Users (CSV)

Import multiple users from CSV file.

**Endpoint:** `POST /users/import/csv`

**Request:** Multipart form data with CSV file

**CSV Format:**
```csv
username,password,realm,display_name,email
alice,secret123,example.com,Alice Smith,alice@example.com
bob,secret456,example.com,Bob Jones,bob@example.com
```

**Response:**
```json
{
  "total": 2,
  "successful": 2,
  "failed": 0,
  "errors": []
}
```

**Status Codes:**
- `200 OK` - Import completed (check result for details)
- `400 Bad Request` - Invalid CSV file

#### Bulk Import Users (JSON)

Import multiple users from JSON array.

**Endpoint:** `POST /users/import/json`

**Request Body:**
```json
[
  {
    "username": "alice",
    "password": "secret123",
    "realm": "example.com",
    "display_name": "Alice Smith",
    "email": "alice@example.com"
  },
  {
    "username": "bob",
    "password": "secret456",
    "realm": "example.com",
    "display_name": "Bob Jones",
    "email": "bob@example.com"
  }
]
```

**Response:** Same as CSV import

**Status Codes:**
- `200 OK` - Import completed
- `400 Bad Request` - Invalid JSON

---

### Role Management

#### List Roles

List all available roles.

**Endpoint:** `GET /roles`

**Response:**
```json
{
  "roles": [
    {
      "id": "a0000000-0000-0000-0000-000000000001",
      "name": "administrator",
      "description": "Full system access with all permissions",
      "permissions": [
        "user:read",
        "user:create",
        "user:update",
        "user:delete",
        "call:read",
        "call:create",
        "system:config"
      ],
      "is_system": true
    }
  ],
  "total": 3
}
```

**Status Codes:**
- `200 OK` - List returned successfully

#### Get Role

Retrieve role by ID.

**Endpoint:** `GET /roles/:id`

**Response:** Single role object

**Status Codes:**
- `200 OK` - Role found
- `404 Not Found` - Role does not exist

#### Create Role

Create a custom role.

**Endpoint:** `POST /roles`

**Request Body:**
```json
{
  "name": "supervisor",
  "description": "Team supervisor with monitoring access",
  "permissions": [
    "call:read",
    "call:terminate",
    "cdr:read",
    "cdr:export"
  ]
}
```

**Response:** Created role object

**Status Codes:**
- `201 Created` - Role created successfully
- `400 Bad Request` - Invalid request
- `409 Conflict` - Role name already exists

#### Update Role

Update role information (non-system roles only).

**Endpoint:** `PUT /roles/:id`

**Request Body:**
```json
{
  "name": "supervisor",
  "description": "Updated description",
  "permissions": [
    "call:read",
    "call:terminate",
    "cdr:read"
  ]
}
```

**Response:** Updated role object

**Status Codes:**
- `200 OK` - Role updated successfully
- `403 Forbidden` - Cannot update system role
- `404 Not Found` - Role does not exist

#### Delete Role

Delete a custom role (non-system roles only).

**Endpoint:** `DELETE /roles/:id`

**Response:**
```json
{
  "message": "Role deleted successfully"
}
```

**Status Codes:**
- `200 OK` - Role deleted successfully
- `403 Forbidden` - Cannot delete system role
- `404 Not Found` - Role does not exist

---

### Call Management

#### List Active Calls

Get list of currently active calls.

**Endpoint:** `GET /calls`

**Response:**
```json
{
  "calls": [
    {
      "call_id": "abc123@example.com",
      "caller": "sip:alice@example.com",
      "callee": "sip:bob@example.com",
      "state": "Established",
      "start_time": "2025-11-06T12:00:00Z",
      "duration_seconds": 120
    }
  ],
  "total": 1
}
```

**Status Codes:**
- `200 OK` - List returned successfully

#### Get Call Details

Get details of a specific call.

**Endpoint:** `GET /calls/:call_id`

**Response:**
```json
{
  "call_id": "abc123@example.com",
  "caller": "sip:alice@example.com",
  "callee": "sip:bob@example.com",
  "state": "Established",
  "start_time": "2025-11-06T12:00:00Z",
  "answer_time": "2025-11-06T12:00:05Z",
  "duration_seconds": 120,
  "codec": "PCMU",
  "caller_ip": "192.168.1.100",
  "callee_ip": "192.168.1.101"
}
```

**Status Codes:**
- `200 OK` - Call found
- `404 Not Found` - Call does not exist

#### Terminate Call

Forcefully terminate an active call.

**Endpoint:** `POST /calls/:call_id/terminate`

**Response:**
```json
{
  "message": "Call terminated successfully"
}
```

**Status Codes:**
- `200 OK` - Call terminated successfully
- `404 Not Found` - Call does not exist

---

### CDR (Call Detail Records)

#### List CDRs

List call detail records with optional filtering.

**Endpoint:** `GET /cdrs`

**Query Parameters:**
- `caller` (optional) - Filter by caller URI
- `callee` (optional) - Filter by callee URI
- `direction` (optional) - Filter by direction (Inbound/Outbound/Internal)
- `status` (optional) - Filter by status
- `start_time_from` (optional) - Filter by start time (ISO 8601)
- `start_time_to` (optional) - Filter by start time (ISO 8601)
- `limit` (optional) - Number of records to return (default: 100, max: 10000)
- `offset` (optional) - Number of records to skip (default: 0)

**Response:**
```json
{
  "cdrs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "call_id": "abc123@example.com",
      "caller": "sip:alice@example.com",
      "callee": "sip:bob@example.com",
      "direction": "Internal",
      "start_time": "2025-11-06T12:00:00Z",
      "answer_time": "2025-11-06T12:00:05Z",
      "end_time": "2025-11-06T12:02:05Z",
      "setup_duration_ms": 5000,
      "call_duration_ms": 120000,
      "total_duration_ms": 125000,
      "status": "Completed",
      "codec": "PCMU"
    }
  ],
  "total": 1,
  "limit": 100,
  "offset": 0
}
```

**Status Codes:**
- `200 OK` - List returned successfully

#### Get CDR by ID

Retrieve a specific call detail record.

**Endpoint:** `GET /cdrs/:id`

**Response:** Single CDR object

**Status Codes:**
- `200 OK` - CDR found
- `404 Not Found` - CDR does not exist

#### Get CDR by Call-ID

Retrieve CDR by SIP Call-ID.

**Endpoint:** `GET /cdrs/call-id/:call_id`

**Response:** Single CDR object

**Status Codes:**
- `200 OK` - CDR found
- `404 Not Found` - CDR does not exist

#### Export CDRs (CSV)

Export call detail records as CSV.

**Endpoint:** `GET /cdrs/export/csv`

**Query Parameters:** Same as List CDRs

**Response:** CSV file download

**Status Codes:**
- `200 OK` - CSV file returned

#### Export CDRs (JSON)

Export call detail records as JSON.

**Endpoint:** `GET /cdrs/export/json`

**Query Parameters:** Same as List CDRs

**Response:** JSON array of CDR objects

**Status Codes:**
- `200 OK` - JSON data returned

---

### Monitoring

#### System Metrics

Get Prometheus-format metrics.

**Endpoint:** `GET /metrics`

**Response:** Prometheus text format

**Metrics Include:**
- `yakyak_active_calls` - Number of active calls
- `yakyak_registered_users` - Number of registered SIP endpoints
- `yakyak_total_calls` - Total calls processed (counter)
- `yakyak_call_duration_seconds` - Call duration histogram

**Status Codes:**
- `200 OK` - Metrics returned

#### WebSocket Events

Real-time system events via WebSocket.

**Endpoint:** `WS /ws`

**Event Types:**
- `call.started` - New call initiated
- `call.answered` - Call answered
- `call.ended` - Call terminated
- `user.registered` - User registered (SIP REGISTER)
- `user.unregistered` - User unregistered

**Event Format:**
```json
{
  "type": "call.started",
  "timestamp": "2025-11-06T12:00:00Z",
  "data": {
    "call_id": "abc123@example.com",
    "caller": "sip:alice@example.com",
    "callee": "sip:bob@example.com"
  }
}
```

---

## Error Responses

All error responses follow this format:

```json
{
  "error": "Error description",
  "details": "Additional error details (optional)"
}
```

**Common Status Codes:**
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists
- `500 Internal Server Error` - Server error

---

## Rate Limiting

**Note**: Rate limiting is not currently implemented but will be added in a future release.

---

## Examples

### Create User with cURL

```bash
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "secret123",
    "realm": "example.com",
    "display_name": "Alice Smith",
    "email": "alice@example.com"
  }'
```

### Export CDRs as CSV

```bash
curl -X GET "http://localhost:8080/cdrs/export/csv?start_time_from=2025-11-01T00:00:00Z&start_time_to=2025-11-06T23:59:59Z" \
  -o cdrs.csv
```

### Bulk Import Users

```bash
curl -X POST http://localhost:8080/users/import/csv \
  -F "file=@users.csv"
```

### Subscribe to WebSocket Events

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data.type, data.data);
};
```

---

## See Also

- [Database Schema Documentation](DATABASE_SCHEMA.md)
- [Authentication Documentation](../AUTH.md)
- [Call Flow Documentation](../CALL_FLOW.md)
