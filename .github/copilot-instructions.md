# A2S Rust Library
A2S-rs is a Rust library implementation of [Source A2S Queries](https://developer.valvesoftware.com/wiki/Server_queries) for querying Source engine game servers (Counter-Strike, Team Fortress 2, etc.). It supports both synchronous and asynchronous operations via feature flags.

**Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

## Working Effectively

### Bootstrap and Build
- Install Rust toolchain (if not available): Use the system's package manager or rustup
- **Initial build**: `cargo build` - takes ~4 seconds with dependency downloads. NEVER CANCEL. Set timeout to 30+ seconds.
- **Subsequent builds**: `cargo build` - takes ~0.1 seconds after dependencies cached
- **Release build**: `cargo build --release` - takes ~5 seconds. NEVER CANCEL. Set timeout to 15+ seconds.
- **Build with all features**: `cargo build --all-features` - takes ~4.5 seconds. NEVER CANCEL. Set timeout to 15+ seconds.
- **Quick syntax check**: `cargo check` - takes ~1 second after initial build

### Testing
- **Run tests**: `cargo test` - tests will FAIL in sandboxed environments due to network requirements
- **Run async tests**: `cargo test --features async` - also requires network connectivity
- **Test timing**: Tests complete in ~1-5 seconds but fail due to network restrictions
- **CRITICAL**: Tests require real game server connectivity and will fail with "No address associated with hostname" errors in restricted environments - this is expected behavior

### Linting and Code Quality  
- **Run clippy**: `cargo clippy` - takes ~1 second. NEVER CANCEL. Set timeout to 10+ seconds.
- **Check formatting**: `cargo fmt --check` - takes ~0.3 seconds
- **Fix formatting**: `cargo fmt` - takes ~0.3 seconds
- **ALWAYS** run `cargo fmt` and `cargo clippy` before committing - the code has known formatting issues

### Documentation
- **Build docs**: `cargo doc` - takes ~4 seconds. NEVER CANCEL. Set timeout to 15+ seconds.
- **Open docs**: `cargo doc --open` - generates and opens documentation in browser (if GUI available)

## Validation Scenarios

### Basic Library Functionality
**ALWAYS test these scenarios after making changes to validate the library works:**

1. **Client Creation Test** (works offline):
```rust
use a2s::A2SClient;
use std::time::Duration;

// Test synchronous client creation
let mut client = A2SClient::new().unwrap();

// Test configuration methods
client.set_timeout(Duration::from_secs(10)).unwrap();
client.max_size(2000);
client.app_id(730); // CS:GO app ID

// Test async client creation (with async feature)
#[cfg(feature = "async")]
let mut client = A2SClient::new().await.unwrap();
```

2. **Offline API Validation** (validates API exists, expects network errors):
```rust
// These calls will fail with network errors but prove the API is functional
let _info = client.info("test.invalid:27015"); // Expected: network error
let _players = client.players("test.invalid:27015"); // Expected: network error  
let _rules = client.rules("test.invalid:27015"); // Expected: network error
```

3. **Feature Flag Validation**:
- `cargo build` - default synchronous build
- `cargo build --features async` - async support
- `cargo build --features serialization` - serde support
- `cargo build --all-features` - complete functionality

### Network Testing (requires real game servers)
**These tests only work with actual game server connectivity:**
- Query live Source engine servers (CS:GO, TF2, etc.) on port 27015
- Test server info, player list, and rules queries
- Validate async concurrent queries work properly
- Test multipacket response handling

## Library Features

### Core APIs
- **A2SClient::new()** - Create client for server queries (sync/async variants)
- **client.info(address)** - Get server information (name, map, players, etc.)
- **client.players(address)** - Get current player list with scores and connection time
- **client.rules(address)** - Get server configuration/rules (cvars)
- **client.set_timeout(duration)** - Configure query timeout (default: 5 seconds)
- **client.max_size(size)** - Set maximum packet size (default: 1400 bytes)
- **client.app_id(id)** - Set Steam application ID for queries

### Feature Flags
- **Default**: Synchronous-only operation with std::net
- **async**: Enables tokio-based async operations
- **serialization**: Adds serde Serialize/Deserialize support

### Response Types
- **Info**: Server name, map, player count, game type, OS, VAC status
- **Players**: Vector of player info (name, score, connection duration)
- **Rules**: Vector of server rules/cvars (name-value pairs)

## Common File Locations

### Source Files
```
src/
├── lib.rs           # Main client implementation and packet handling
├── info.rs          # Server info queries and response parsing
├── players.rs       # Player list queries and response parsing  
├── rules.rs         # Server rules queries and response parsing
└── errors.rs        # Error types and Result definitions
```

### Test Files
```
tests/
├── info_test.rs     # Server info query tests (require network)
├── players_test.rs  # Player query tests (require network)
├── rules_test.rs    # Rules query tests (require network)
└── async_test.rs    # Async operation tests (require network)
```

### Key Configuration
```
Cargo.toml           # Dependencies and feature definitions
README.md           # Basic library information  
LICENSE             # MIT license
.gitignore          # Standard Rust gitignore
```

## Build and Test Timing Expectations

### Build Times (with appropriate timeout values)
- **Initial build**: 4+ seconds (downloading dependencies) - set timeout to 30+ seconds. NEVER CANCEL.
- **Incremental build**: 0.1 seconds - set timeout to 10+ seconds  
- **Release build**: 5+ seconds - set timeout to 15+ seconds. NEVER CANCEL.
- **All features build**: 4.5+ seconds - set timeout to 15+ seconds. NEVER CANCEL.
- **Documentation**: 4+ seconds - set timeout to 15+ seconds. NEVER CANCEL.
- **Linting (clippy)**: 1+ seconds - set timeout to 10+ seconds
- **Formatting check**: 0.2 seconds - set timeout to 5+ seconds

### Test Expectations
- Tests require network connectivity to game servers
- In sandboxed environments, expect "No address associated with hostname" failures
- This is normal behavior - tests work with real game server access
- Test timing: 1-5 seconds when they can run successfully

## Critical Notes
- **NEVER CANCEL** any build or compilation command - Rust builds can take time
- **ALWAYS** format code with `cargo fmt` before committing
- **ALWAYS** run `cargo clippy` to check for linting issues
- Tests require network access to game servers and will fail in restricted environments
- The library implements the Source A2S query protocol, NOT Goldsource
- Default timeout is 5 seconds for server queries
- Maximum packet size defaults to 1400 bytes for UDP queries
- Supports both IPv4 and IPv6 addresses for server queries

## Common Command Outputs
The following are outputs from frequently run commands. Reference them instead of viewing, searching, or running bash commands to save time.

### Repository root directory
```
ls -la [repo-root]
.github/                  # GitHub configuration (including this file)
src/                      # Main library source code
tests/                    # Integration tests (require network)
target/                   # Build artifacts (created after build)
Cargo.toml               # Project configuration and dependencies
Cargo.lock               # Dependency lock file (created after build)
README.md                # Basic library documentation
LICENSE                  # MIT license
.gitignore               # Git ignore rules
```

### Source directory structure
```
ls -la src/
errors.rs                # Error types and Result definitions
info.rs                  # Server info queries and response parsing
lib.rs                   # Main client implementation and packet handling
players.rs               # Player list queries and response parsing
rules.rs                 # Server rules queries and response parsing
```

### Test directory structure  
```
ls -la tests/
async_test.rs            # Async operation tests (require network)
info_test.rs             # Server info query tests (require network)
players_test.rs          # Player query tests (require network)
rules_test.rs            # Rules query tests (require network)
```