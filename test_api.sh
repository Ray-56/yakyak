#!/bin/bash

# Test script for YakYak User Management API

BASE_URL="http://localhost:8080"

echo "=== YakYak API Test Suite ==="
echo ""

# Wait for server to start
sleep 2

# Test 1: Health check
echo "Test 1: Health Check"
curl -s "$BASE_URL/health" | jq .
echo ""

# Test 2: Create user alice
echo "Test 2: Create user 'alice'"
curl -s -X POST "$BASE_URL/users" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "secret123",
    "realm": "localhost",
    "display_name": "Alice Smith",
    "email": "alice@example.com"
  }' | jq .
echo ""

# Test 3: Create user bob
echo "Test 3: Create user 'bob'"
curl -s -X POST "$BASE_URL/users" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "bob",
    "password": "secret456",
    "realm": "localhost",
    "display_name": "Bob Jones",
    "email": "bob@example.com"
  }' | jq .
echo ""

# Test 4: List all users
echo "Test 4: List all users"
curl -s "$BASE_URL/users?limit=10&offset=0" | jq .
echo ""

# Test 5: Get user by ID
echo "Test 5: Get user by ID (1)"
curl -s "$BASE_URL/users/1" | jq .
echo ""

# Test 6: Get user by username
echo "Test 6: Get user by username (alice)"
curl -s "$BASE_URL/users/username/alice" | jq .
echo ""

# Test 7: Update user
echo "Test 7: Update user display name"
curl -s -X PUT "$BASE_URL/users/1" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Alice Cooper",
    "email": "alice.cooper@example.com"
  }' | jq .
echo ""

# Test 8: Change password
echo "Test 8: Change user password"
curl -s -X POST "$BASE_URL/users/1/password" \
  -H "Content-Type: application/json" \
  -d '{
    "old_password": "secret123",
    "new_password": "newsecret123"
  }' | jq .
echo ""

# Test 9: Disable user
echo "Test 9: Disable user"
curl -s -X PUT "$BASE_URL/users/2/enabled/false" | jq .
echo ""

# Test 10: Enable user
echo "Test 10: Enable user"
curl -s -X PUT "$BASE_URL/users/2/enabled/true" | jq .
echo ""

# Test 11: List users filtered by realm
echo "Test 11: List users by realm"
curl -s "$BASE_URL/users?realm=localhost&limit=10" | jq .
echo ""

echo "=== Test Suite Complete ==="
