#!/bin/bash
#
# Generate self-signed certificates for TLS testing
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CERT_DIR="$PROJECT_ROOT/certs"

echo "Generating self-signed TLS certificates for testing..."

# Create certs directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Generate private key
echo "Generating private key..."
openssl genrsa -out "$CERT_DIR/server.key" 2048

# Generate self-signed certificate
echo "Generating self-signed certificate..."
openssl req -new -x509 -key "$CERT_DIR/server.key" \
    -out "$CERT_DIR/server.crt" \
    -days 365 \
    -subj "/C=US/ST=California/L=San Francisco/O=YakYak PBX/OU=Development/CN=localhost"

echo ""
echo "✅ Certificates generated successfully!"
echo "   Certificate: $CERT_DIR/server.crt"
echo "   Private Key: $CERT_DIR/server.key"
echo ""
echo "⚠️  WARNING: These are self-signed certificates for TESTING ONLY."
echo "   Do NOT use these certificates in production!"
echo ""
