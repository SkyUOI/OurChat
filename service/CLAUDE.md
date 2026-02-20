# CLAUDE.md - Service

This file provides guidance to Claude Code (claude.ai/code) when working with the **service** portion of this repository.

## Directory Overview

The `service/` directory contains all Protocol Buffer (`.proto`) definitions for the OurChat application's gRPC services. These definitions are shared between the Rust server and Flutter/Dart client to ensure type-safe communication.

## Directory Structure

`
service/
├── auth/              # Authentication and authorization services
├── basic/             # Basic services (no auth required)
├── ourchat/           # Main OurChat service (authenticated)
└── server_manage/     # Server management and monitoring
`

### Top-Level Services

#### `auth/` - Authentication Service

**Package:** `service.auth.v1`

Services for user registration, login, and email verification. These endpoints do not require JWT authentication.

#### `basic/` - Basic Service

**Package:** `service.basic.v1`

Basic server information endpoints that do not require authentication.

#### `ourchat/` - Main OurChat Service

**Package:** `service.ourchat.v1`

Core chat functionality requiring JWT authentication. All endpoints require `Authorization: Bearer <token>` header.

#### `server_manage/` - Server Management Service

**Package:** `service.server_manage.v1`

Administrative endpoints for server management and monitoring.

## Protocol Buffer Conventions

### File Organization

Each service follows this structure:

`service/<service>/<feature>/v1/<feature>.proto`

Examples:

- `service/auth/register/v1/register.proto`
- `service/ourchat/session/new_session/v1/session.proto`

### Naming Conventions

1. **Package names:** `service.<service>.v1`
2. **Message names:** `<Action>Request` / `<Action>Response`
3. **RPC names:** `<CamelCaseAction>`

### Message Patterns

**Response messages** typically contain:

- Result data or success confirmation
- Error details (via gRPC status)

## Code Generation

### For Rust (Server)

The protobuf files are compiled to Rust code via `prost` and `tonic` build scripts.

**Output location:** `server/pb/src/generated/`

### For Dart (Client)

Run the protobuf generation script when `.proto` files change:

```bash
python script/generate.pb.dart.py
```

**Output location:** `client/lib/libgrpc/generated/`

### For gRPC-Web (Web Client)

```bash
python script/generate_grpc_web.py
```

## Development Workflow

### Adding a New RPC

1. **Create the proto file** in the appropriate directory:
   `service/<service>/<feature>/v1/<feature>.proto`

2. **Define messages and service:**

   ```protobuf
   syntax = "proto3";

   package service.ourchat.v1;

   message MyFeatureRequest {
     string param = 1;
   }

   message MyFeatureResponse {
     string result = 1;
   }
   ```

3. **Add to parent service proto** (if needed):

   ```protobuf
   import "service/ourchat/my_feature/v1/my_feature.proto";

   service OurChatService {
       rpc MyFeature(my_feature.v1.MyFeatureRequest) returns (my_feature.v1.MyFeatureResponse);
   }
   ```

4. **Generate code:**
   - Server: Automatically rebuilt via Cargo
   - Client: `python script/generate.pb.dart.py`

5. **Implement server handler** in `server/src/process/`

### Adding New Fields

When adding new fields to existing messages:

1. Add field with a new tag number (never reuse tag numbers)
2. Regenerate code for both server and client

## Common Patterns

### Timestamp Fields

Use `google.protobuf.Timestamp` for all timestamp fields:

```protobuf
import "google/protobuf/timestamp.proto";

message MyMessage {
  google.protobuf.Timestamp created_at = 1;
}
```

### Duration Fields

Use `google.protobuf.Duration` for time durations:

```protobuf
import "google/protobuf/duration.proto";

message MyMessage {
  google.protobuf.Duration timeout = 1;
}
```

## Important Notes

- **Document all RPCs** with comments explaining their purpose
- **Follow semantic versioning** in the `v1` package paths
- **Keep messages simple** - avoid deeply nested structures when possible

## Related Files

- **`server/pb/build.rs`** - Rust protobuf build configuration
- **`script/generate.pb.dart.py`** - Dart code generation script
- **`script/generate_grpc_web.py`** - gRPC-Web generation script
