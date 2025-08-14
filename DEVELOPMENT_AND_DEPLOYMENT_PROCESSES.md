# Automation Nation - Development and Deployment Processes

## 📋 Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Development Workflow](#development-workflow)
3. [Code Standards and Guidelines](#code-standards-and-guidelines)
4. [Testing Strategies](#testing-strategies)
5. [Build and Release Process](#build-and-release-process)
6. [Enterprise Deployment](#enterprise-deployment)
7. [Cloud Deployment Strategies](#cloud-deployment-strategies)
8. [Monitoring and Observability](#monitoring-and-observability)
9. [Security Procedures](#security-procedures)
10. [Maintenance and Operations](#maintenance-and-operations)

---

## 🛠️ Development Environment Setup

### Prerequisites

**System Requirements:**
```bash
# Operating System Support
- Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+, Debian 11+)
- macOS (10.15+)
- Windows (WSL2 with Ubuntu 20.04+)

# Minimum Hardware
- CPU: 4 cores (8 recommended)
- RAM: 8GB (16GB recommended)
- Storage: 20GB free space (SSD recommended)
- Network: Broadband internet connection
```

**Required Software:**
```bash
# Core Development Tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup default stable
rustup component add clippy rustfmt

# Container Runtime (choose one or more)
# Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Podman (alternative to Docker)
sudo apt install podman podman-compose  # Ubuntu/Debian
sudo dnf install podman podman-compose  # Fedora/RHEL

# Database Tools
sudo apt install postgresql-client sqlite3
cargo install sqlx-cli --features postgres,sqlite

# Testing Tools
sudo apt install bats  # Bash testing framework
npm install -g newman  # API testing
```

### Development Environment Configuration

**Rust Development Setup:**
```bash
# Clone the repository
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation

# Install Rust dependencies
cargo build --release

# Install development tools
cargo install cargo-watch cargo-audit cargo-tarpaulin
rustup component add llvm-tools-preview

# Setup Git hooks
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# Configure environment
cp .env.template .env.development
# Edit .env.development with your local settings
```

**Database Setup:**
```bash
# PostgreSQL (recommended for development)
sudo -u postgres createdb automation_nation_dev
sudo -u postgres createuser automation_dev --pwprompt

# Update .env.development
DATABASE_URL=postgresql://automation_dev:password@localhost:5432/automation_nation_dev

# Run migrations
cargo sqlx migrate run

# SQLite (lightweight alternative)
DATABASE_URL=sqlite:///tmp/automation_nation_dev.db
cargo sqlx database create
cargo sqlx migrate run
```

**Docker Development Stack:**
```yaml
# docker-compose.development.yml
version: '3.8'

services:
  postgres-dev:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: automation_nation_dev
      POSTGRES_USER: automation_dev
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_dev_data:/var/lib/postgresql/data

  redis-dev:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --requirepass dev_password

  netbox-dev:
    image: netboxcommunity/netbox:latest
    environment:
      SUPERUSER_NAME: admin
      SUPERUSER_PASSWORD: admin
      SUPERUSER_EMAIL: admin@example.com
      SECRET_KEY: development_secret_key
    ports:
      - "8080:8080"
    depends_on:
      - postgres-dev
      - redis-dev

volumes:
  postgres_dev_data:
```

### IDE Configuration

**Visual Studio Code Setup:**
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.imports.granularity.group": "module",
    "rust-analyzer.completion.addCallArgumentSnippets": true,
    "files.associations": {
        "*.rs": "rust"
    },
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true
    }
}

// .vscode/extensions.json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "ms-vscode.vscode-json",
        "redhat.vscode-yaml",
        "ms-vscode-remote.remote-containers"
    ]
}
```

**IntelliJ IDEA Setup:**
```bash
# Install Rust plugin
# Go to File -> Settings -> Plugins -> Marketplace
# Search for "Rust" and install

# Configure code style
# File -> Settings -> Editor -> Code Style -> Rust
# Set line length to 100
# Enable "Use tabs" for indentation
```

---

## 🔄 Development Workflow

### Git Workflow

**Branch Strategy:**
```bash
# Main branches
main            # Production-ready code
develop         # Integration branch for features
release/v1.x.x  # Release preparation branches

# Feature branches
feature/ISSUE-123-add-container-support
feature/ISSUE-456-implement-sso

# Hotfix branches
hotfix/ISSUE-789-fix-critical-security-bug

# Branch naming convention
git checkout -b feature/ISSUE-123-short-description
git checkout -b bugfix/ISSUE-456-fix-description
git checkout -b hotfix/ISSUE-789-critical-fix
```

**Commit Message Standards:**
```bash
# Format: <type>(<scope>): <subject>
#
# Types: feat, fix, docs, style, refactor, test, chore
# Scope: api, container, auth, db, test, docs
# Subject: imperative mood, no period, max 50 chars

# Examples
feat(api): add container deployment endpoint
fix(auth): resolve JWT token expiration issue
docs(readme): update installation instructions
test(container): add Docker runtime integration tests
refactor(db): optimize query performance
chore(deps): update Rust dependencies
```

**Pull Request Process:**
```bash
# 1. Create feature branch
git checkout -b feature/new-feature
git push -u origin feature/new-feature

# 2. Make changes and commit
git add .
git commit -m "feat(scope): implement new feature"

# 3. Push changes
git push origin feature/new-feature

# 4. Create pull request
# - Use PR template
# - Link to issue
# - Add reviewers
# - Ensure CI passes

# 5. Code review process
# - Address review comments
# - Update documentation
# - Ensure tests pass

# 6. Merge to develop
git checkout develop
git merge --no-ff feature/new-feature
git tag -a v1.2.3 -m "Release version 1.2.3"
git push origin develop --tags
```

### Code Review Guidelines

**Review Checklist:**
```markdown
## Code Quality
- [ ] Code follows Rust best practices
- [ ] Functions are well-documented
- [ ] Error handling is comprehensive
- [ ] No unwrap() calls in production code
- [ ] Memory safety considerations addressed

## Testing
- [ ] Unit tests cover new functionality
- [ ] Integration tests pass
- [ ] Performance impact assessed
- [ ] Security implications reviewed

## Documentation
- [ ] API changes documented
- [ ] Configuration changes noted
- [ ] Deployment instructions updated
- [ ] Changelog entry added

## Security
- [ ] Input validation implemented
- [ ] Authentication/authorization checked
- [ ] SQL injection prevention
- [ ] XSS prevention measures
- [ ] Sensitive data handling reviewed
```

### Continuous Integration

**GitHub Actions Workflow:**
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: automation_nation_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        override: true
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --features postgres
    
    - name: Setup database
      run: |
        export DATABASE_URL=postgresql://postgres:test_password@localhost:5432/automation_nation_test
        sqlx database create
        sqlx migrate run
      env:
        DATABASE_URL: postgresql://postgres:test_password@localhost:5432/automation_nation_test
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test --verbose
      env:
        DATABASE_URL: postgresql://postgres:test_password@localhost:5432/automation_nation_test
    
    - name: Run integration tests
      run: |
        chmod +x ./comprehensive_test_suite.sh
        ./comprehensive_test_suite.sh --ci
    
    - name: Security audit
      run: cargo audit
    
    - name: Generate coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Xml
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./cobertura.xml
        flags: unittests
        name: codecov-umbrella

  build:
    runs-on: ubuntu-latest
    needs: test
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Build release
      run: cargo build --release
    
    - name: Build Docker image
      run: |
        docker build -t automation-nation:${{ github.sha }} .
        docker tag automation-nation:${{ github.sha }} automation-nation:latest
    
    - name: Run security scan
      run: |
        docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
          -v $(pwd):/src aquasec/trivy fs --security-checks vuln /src
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: automation-nation-${{ github.sha }}
        path: |
          target/release/web_server
          target/release/ci_runner
          target/release/comprehensive_test_runner
```

---

## 📏 Code Standards and Guidelines

### Rust Coding Standards

**Project Structure:**
```
src/
├── lib.rs                      # Public API exports
├── config.rs                   # Configuration management
├── types.rs                    # Common type definitions
├── error.rs                    # Error types and handling
├── 
├── api/                        # Web API layer
│   ├── mod.rs
│   ├── handlers/               # HTTP request handlers
│   ├── middleware/             # Custom middleware
│   ├── routes.rs               # Route definitions
│   └── responses.rs            # Response types
├── 
├── core/                       # Business logic
│   ├── mod.rs
│   ├── auth/                   # Authentication/authorization
│   ├── container/              # Container management
│   ├── system/                 # System profiling
│   └── deployment/             # Deployment management
├── 
├── db/                         # Database layer
│   ├── mod.rs
│   ├── models.rs               # Database models
│   ├── migrations/             # Database migrations
│   └── queries.rs              # SQL queries
├── 
├── external/                   # External integrations
│   ├── mod.rs
│   ├── docker.rs               # Docker API
│   ├── github.rs               # GitHub API
│   └── sso.rs                  # SSO providers
├── 
└── utils/                      # Utility functions
    ├── mod.rs
    ├── crypto.rs               # Cryptographic utilities
    ├── validation.rs           # Input validation
    └── formatting.rs           # Output formatting
```

**Error Handling Patterns:**
```rust
// Use custom error types with context
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("Container not found: {id}")]
    NotFound { id: String },
    
    #[error("Container runtime error: {message}")]
    RuntimeError { message: String },
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

// Result type alias for consistency
pub type ContainerResult<T> = std::result::Result<T, ContainerError>;

// Error handling in functions
pub async fn deploy_container(config: &DeploymentConfig) -> ContainerResult<Container> {
    let runtime = get_container_runtime()
        .await
        .context("Failed to initialize container runtime")?;
    
    runtime.deploy(config)
        .await
        .with_context(|| format!("Failed to deploy container {}", config.name))
}
```

**Async Programming Guidelines:**
```rust
// Use async/await consistently
pub async fn process_deployment_queue(&self) -> Result<()> {
    let deployments = self.get_pending_deployments().await?;
    
    // Process deployments concurrently with limited parallelism
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent deployments
    let tasks: Vec<_> = deployments
        .into_iter()
        .map(|deployment| {
            let semaphore = semaphore.clone();
            let processor = self.clone();
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                processor.deploy_container(deployment).await
            })
        })
        .collect();
    
    // Wait for all deployments to complete
    let results = futures::future::try_join_all(tasks).await?;
    
    Ok(())
}

// Channel-based communication patterns
pub async fn start_deployment_worker(&self) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<DeploymentRequest>(100);
    
    // Spawn worker task
    let processor = self.clone();
    tokio::spawn(async move {
        while let Some(request) = rx.recv().await {
            if let Err(e) = processor.handle_deployment(request).await {
                error!("Deployment failed: {}", e);
            }
        }
    });
    
    // Store sender for use by API handlers
    self.deployment_sender.store(Some(tx));
    
    Ok(())
}
```

**Database Patterns:**
```rust
// Use transactions for consistency
pub async fn create_user_with_roles(
    &self,
    user_data: &CreateUserRequest,
    roles: &[String],
) -> Result<User> {
    let mut tx = self.pool.begin().await?;
    
    // Create user
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash, status)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, status, created_at
        "#,
        user_data.username,
        user_data.email,
        user_data.password_hash,
        "active"
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Assign roles
    for role_name in roles {
        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, role_name)
            VALUES ($1, $2)
            "#,
            user.id,
            role_name
        )
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    
    info!(user_id = %user.id, username = %user.username, "User created successfully");
    
    Ok(user)
}

// Connection pooling configuration
pub async fn create_database_pool(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
        .context("Failed to create database connection pool")
}
```

### Testing Standards

**Unit Testing Patterns:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    use pretty_assertions::assert_eq;
    
    #[tokio::test]
    async fn test_container_deployment_success() {
        // Arrange
        let config = DeploymentConfig::builder()
            .name("test-container")
            .image("nginx:latest")
            .port(80)
            .build();
        
        let runtime = MockContainerRuntime::new()
            .expect_deploy()
            .with(predicate::eq(config.clone()))
            .times(1)
            .returning(|_| Ok(Container::new("container-123")));
        
        let manager = ContainerManager::new(Box::new(runtime));
        
        // Act
        let result = manager.deploy_container(&config).await;
        
        // Assert
        assert!(result.is_ok());
        let container = result.unwrap();
        assert_eq!(container.id, "container-123");
    }
    
    #[tokio::test]
    async fn test_container_deployment_failure() {
        // Arrange
        let config = DeploymentConfig::default();
        let runtime = MockContainerRuntime::new()
            .expect_deploy()
            .returning(|_| Err(ContainerError::RuntimeError {
                message: "Image not found".to_string()
            }));
        
        let manager = ContainerManager::new(Box::new(runtime));
        
        // Act
        let result = manager.deploy_container(&config).await;
        
        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ContainerError::RuntimeError { message } => {
                assert_eq!(message, "Image not found");
            }
            _ => panic!("Expected RuntimeError"),
        }
    }
}
```

**Integration Testing:**
```rust
// Integration test setup
#[tokio::test]
async fn integration_test_full_deployment_workflow() {
    // Setup test database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "sqlite::memory:".to_string());
    
    let pool = PgPool::connect(&database_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // Setup test server
    let app_state = WebAppState::new(pool.clone()).await.unwrap();
    let app = create_web_routes().with_state(app_state);
    
    // Create test client
    let client = TestClient::new(app);
    
    // Test deployment workflow
    let deployment_request = json!({
        "name": "test-app",
        "image": "nginx:latest",
        "port": 80
    });
    
    // Create deployment
    let response = client
        .post("/api/deployments")
        .json(&deployment_request)
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let deployment: Deployment = response.json().await;
    assert_eq!(deployment.name, "test-app");
    
    // Get deployment status
    let response = client
        .get(&format!("/api/deployments/{}", deployment.id))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Cleanup
    sqlx::query!("DELETE FROM deployments WHERE id = $1", deployment.id)
        .execute(&pool)
        .await
        .unwrap();
}
```

---

## 🧪 Testing Strategies

### Test Pyramid Structure

```
                    ╔══════════════════╗
                    ║   E2E Tests      ║  <- Selenium, API tests
                    ║   (Slow, Broad)  ║
                    ╚══════════════════╝
               ╔══════════════════════════════╗
               ║     Integration Tests        ║  <- Component integration
               ║     (Medium, Focused)        ║
               ╚══════════════════════════════╝
          ╔═══════════════════════════════════════════╗
          ║              Unit Tests                   ║  <- Fast, isolated
          ║              (Fast, Narrow)               ║
          ╚═══════════════════════════════════════════╝
```

### Testing Framework Configuration

**Cargo.toml Test Dependencies:**
```toml
[dev-dependencies]
tokio-test = "0.4"
pretty_assertions = "1.0"
mockall = "0.11"
axum-test = "14.0"
serde_json = "1.0"
tempfile = "3.0"
criterion = "0.5"
proptest = "1.0"
sqlx-test = "0.7"

# Performance testing
[[bench]]
name = "container_operations"
harness = false

[[bench]]
name = "api_endpoints"
harness = false
```

**Test Configuration:**
```rust
// tests/common/mod.rs - Shared test utilities
use sqlx::{PgPool, Postgres, Transaction};
use tempfile::TempDir;
use tokio::sync::OnceCell;

pub static TEST_DB: OnceCell<PgPool> = OnceCell::const_new();

pub async fn setup_test_db() -> &'static PgPool {
    TEST_DB.get_or_init(|| async {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost:5432/automation_nation_test".to_string());
        
        let pool = PgPool::connect(&database_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        pool
    }).await
}

pub async fn setup_test_transaction() -> Transaction<'static, Postgres> {
    let pool = setup_test_db().await;
    pool.begin().await.unwrap()
}

pub fn setup_test_config() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create test configuration files
    std::fs::write(
        temp_dir.path().join("test.toml"),
        r#"
        [server]
        host = "127.0.0.1"
        port = 0
        
        [database]
        url = "sqlite::memory:"
        max_connections = 5
        
        [auth]
        jwt_secret = "test-secret"
        jwt_expiration = 3600
        "#
    ).unwrap();
    
    temp_dir
}
```

### Performance Testing

**Benchmark Configuration:**
```rust
// benches/container_operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use automation_nation::{ContainerManager, DeploymentConfig};
use tokio::runtime::Runtime;

fn benchmark_container_deployment(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(ContainerManager::new()).unwrap();
    
    let mut group = c.benchmark_group("container_deployment");
    
    for size in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_deployments", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let configs: Vec<_> = (0..*size)
                        .map(|i| DeploymentConfig::builder()
                            .name(format!("test-{}", i))
                            .image("nginx:latest")
                            .build())
                        .collect();
                    
                    let tasks: Vec<_> = configs
                        .into_iter()
                        .map(|config| manager.deploy_container(&config))
                        .collect();
                    
                    black_box(futures::future::try_join_all(tasks).await)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_container_deployment);
criterion_main!(benches);
```

**Load Testing:**
```javascript
// tests/load/api_load_test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';

// Load test configuration
export let options = {
  stages: [
    { duration: '2m', target: 10 },   // Ramp up
    { duration: '5m', target: 50 },   // Stay at 50 users
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests under 500ms
    http_req_failed: ['rate<0.1'],    // Error rate under 10%
  },
};

// Test data
const testUsers = new SharedArray('users', function () {
  return JSON.parse(open('./test_users.json'));
});

const BASE_URL = 'http://localhost:3000';

export function setup() {
  // Setup test data
  return {
    authToken: login(),
  };
}

export default function (data) {
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${data.authToken}`,
    },
  };
  
  // Test system profile endpoint
  let response = http.get(`${BASE_URL}/api/system/profile`, params);
  check(response, {
    'system profile status is 200': (r) => r.status === 200,
    'system profile response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  // Test container runtime listing
  response = http.get(`${BASE_URL}/api/containers/runtimes`, params);
  check(response, {
    'container runtimes status is 200': (r) => r.status === 200,
    'container runtimes response time < 100ms': (r) => r.timings.duration < 100,
  });
  
  // Test deployment creation
  const deploymentConfig = {
    name: `load-test-${__VU}-${__ITER}`,
    image: 'nginx:latest',
    port: 80,
  };
  
  response = http.post(
    `${BASE_URL}/api/deployments`,
    JSON.stringify(deploymentConfig),
    params
  );
  
  check(response, {
    'deployment creation status is 201': (r) => r.status === 201,
    'deployment creation response time < 1s': (r) => r.timings.duration < 1000,
  });
  
  sleep(1);
}

function login() {
  const loginData = {
    username: 'testuser',
    password: 'testpassword',
  };
  
  const response = http.post(
    `${BASE_URL}/api/auth/login`,
    JSON.stringify(loginData),
    {
      headers: { 'Content-Type': 'application/json' },
    }
  );
  
  return JSON.parse(response.body).token;
}

export function teardown(data) {
  // Cleanup test data
}
```

---

## 🏗️ Build and Release Process

### Automated Build Pipeline

**Multi-Stage Docker Build:**
```dockerfile
# Dockerfile.optimized
# Stage 1: Build dependencies
FROM rust:1.75-slim as dependency-builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Stage 2: Build application
FROM dependency-builder as app-builder
COPY src ./src
COPY migrations ./migrations
COPY plugins ./plugins
COPY collect_info.sh ./

# Build application with optimizations
ENV CARGO_TARGET_DIR=/tmp/target
RUN cargo build --release --locked

# Stage 3: Runtime image
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql-client \
    curl \
    jq \
    lshw \
    lsof \
    iproute2 \
    net-tools \
    && rm -rf /var/lib/apt/lists/*

# Create application user
RUN groupadd -r automation && useradd -r -g automation automation

# Copy application artifacts
COPY --from=app-builder /tmp/target/release/web_server /usr/local/bin/
COPY --from=app-builder /tmp/target/release/ci_runner /usr/local/bin/
COPY --from=app-builder /app/collect_info.sh /usr/local/bin/
COPY --from=app-builder /app/plugins /usr/local/share/automation-nation/plugins
COPY --from=app-builder /app/migrations /usr/local/share/automation-nation/migrations

# Set permissions
RUN chmod +x /usr/local/bin/* /usr/local/share/automation-nation/plugins/*
RUN chown -R automation:automation /usr/local/share/automation-nation

# Create data directories
RUN mkdir -p /app/data /app/logs && chown automation:automation /app/data /app/logs

USER automation
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:3000/api/health || exit 1

EXPOSE 3000

# Default command
CMD ["web_server", "serve", "--host", "0.0.0.0", "--port", "3000"]

# Metadata
LABEL org.opencontainers.image.title="Automation Nation"
LABEL org.opencontainers.image.description="Enterprise automation platform"
LABEL org.opencontainers.image.source="https://github.com/nullroute-commits/Automation_nation"
LABEL org.opencontainers.image.licenses="MIT"
```

**Release Automation:**
```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
  workflow_dispatch:
    inputs:
      force_release:
        description: 'Force release even without changes'
        required: false
        default: 'false'

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  check-changes:
    runs-on: ubuntu-latest
    outputs:
      has_changes: ${{ steps.changes.outputs.has_changes }}
      version: ${{ steps.version.outputs.version }}
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Check for changes since last release
      id: changes
      run: |
        LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
        if [ -z "$LAST_TAG" ] || [ "${{ github.event.inputs.force_release }}" == "true" ]; then
          echo "has_changes=true" >> $GITHUB_OUTPUT
        else
          CHANGES=$(git diff --name-only $LAST_TAG..HEAD)
          if [ -n "$CHANGES" ]; then
            echo "has_changes=true" >> $GITHUB_OUTPUT
          else
            echo "has_changes=false" >> $GITHUB_OUTPUT
          fi
        fi
    
    - name: Generate version
      id: version
      run: |
        VERSION=$(date -u +%Y-%m-%dT%H-%M-%S)
        echo "version=$VERSION" >> $GITHUB_OUTPUT

  build-and-test:
    needs: check-changes
    if: needs.check-changes.outputs.has_changes == 'true'
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu]
        include:
          - target: x86_64-unknown-linux-gnu
            arch: amd64
          - target: aarch64-unknown-linux-gnu
            arch: arm64
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Install cross-compilation tools
      if: matrix.target == 'aarch64-unknown-linux-gnu'
      run: |
        sudo apt-get update
        sudo apt-get install -y gcc-aarch64-linux-gnu
        echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> $GITHUB_ENV
    
    - name: Build release binaries
      run: |
        cargo build --release --target ${{ matrix.target }}
        mkdir -p artifacts/${{ matrix.arch }}
        cp target/${{ matrix.target }}/release/web_server artifacts/${{ matrix.arch }}/
        cp target/${{ matrix.target }}/release/ci_runner artifacts/${{ matrix.arch }}/
        cp target/${{ matrix.target }}/release/comprehensive_test_runner artifacts/${{ matrix.arch }}/
    
    - name: Create release archive
      run: |
        cd artifacts/${{ matrix.arch }}
        tar -czf ../../automation-nation-${{ needs.check-changes.outputs.version }}-${{ matrix.arch }}.tar.gz *
    
    - name: Upload build artifacts
      uses: actions/upload-artifact@v3
      with:
        name: automation-nation-${{ matrix.arch }}
        path: automation-nation-${{ needs.check-changes.outputs.version }}-${{ matrix.arch }}.tar.gz

  build-docker:
    needs: [check-changes, build-and-test]
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
    
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=raw,value=${{ needs.check-changes.outputs.version }}
          type=raw,value=latest,enable={{is_default_branch}}
    
    - name: Build and push Docker image
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./Dockerfile.optimized
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  create-release:
    needs: [check-changes, build-and-test, build-docker]
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Download all artifacts
      uses: actions/download-artifact@v3
    
    - name: Generate changelog
      run: |
        echo "# Release ${{ needs.check-changes.outputs.version }}" > CHANGELOG.md
        echo "" >> CHANGELOG.md
        echo "## Changes" >> CHANGELOG.md
        git log --oneline --since="$(git describe --tags --abbrev=0 2>/dev/null | xargs git log -1 --format=%ai)" >> CHANGELOG.md
    
    - name: Create Release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: v${{ needs.check-changes.outputs.version }}
        release_name: Release ${{ needs.check-changes.outputs.version }}
        body_path: CHANGELOG.md
        draft: false
        prerelease: false
    
    - name: Upload release assets
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ steps.create_release.outputs.upload_url }}
        asset_path: ./automation-nation-amd64/automation-nation-${{ needs.check-changes.outputs.version }}-amd64.tar.gz
        asset_name: automation-nation-${{ needs.check-changes.outputs.version }}-amd64.tar.gz
        asset_content_type: application/gzip

  deploy-staging:
    needs: [create-release]
    runs-on: ubuntu-latest
    environment: staging
    
    steps:
    - name: Deploy to staging
      run: |
        echo "Deploying to staging environment..."
        # Add staging deployment logic here
```

---

## 🏢 Enterprise Deployment

### Production Infrastructure Requirements

**Hardware Specifications:**

| Component | Minimum | Recommended | High Availability |
|-----------|---------|-------------|-------------------|
| **Web Application Nodes** | 2 CPU, 4GB RAM | 4 CPU, 8GB RAM | 6 CPU, 16GB RAM |
| **Database Server** | 4 CPU, 8GB RAM | 8 CPU, 16GB RAM | 16 CPU, 32GB RAM |
| **Redis Cache** | 2 CPU, 4GB RAM | 4 CPU, 8GB RAM | 6 CPU, 16GB RAM |
| **Load Balancer** | 2 CPU, 2GB RAM | 4 CPU, 4GB RAM | 6 CPU, 8GB RAM |
| **Monitoring Stack** | 4 CPU, 8GB RAM | 8 CPU, 16GB RAM | 12 CPU, 24GB RAM |
| **Storage** | 100GB SSD | 500GB SSD | 1TB SSD (replicated) |
| **Network** | 1Gbps | 10Gbps | 10Gbps (redundant) |

**Network Architecture:**
```
Internet
    │
    ▼
┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │◄──►│   Load Balancer │  (HA Pair)
│     (Primary)   │    │   (Secondary)   │
└─────────────────┘    └─────────────────┘
          │                       │
          ▼                       ▼
┌─────────────────────────────────────────┐
│            DMZ Network                  │
│         (172.16.1.0/24)                │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Application Tier                │
│         (172.16.2.0/24)                │
│  ┌─────────────┐  ┌─────────────┐      │
│  │   Web App   │  │   Web App   │      │
│  │   Node 1    │  │   Node 2    │      │
│  └─────────────┘  └─────────────┘      │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           Data Tier                     │
│         (172.16.3.0/24)                │
│  ┌─────────────┐  ┌─────────────┐      │
│  │ PostgreSQL  │  │    Redis    │      │
│  │  Primary    │  │   Cluster   │      │
│  └─────────────┘  └─────────────┘      │
│  ┌─────────────┐                       │
│  │ PostgreSQL  │                       │
│  │  Replica    │                       │
│  └─────────────┘                       │
└─────────────────────────────────────────┘
```

### High Availability Configuration

**Docker Swarm Deployment:**
```yaml
# docker-compose.production.yml
version: '3.8'

services:
  automation-nation:
    image: ghcr.io/nullroute-commits/automation_nation:latest
    deploy:
      replicas: 3
      placement:
        constraints:
          - node.role == worker
        preferences:
          - spread: node.labels.zone
      update_config:
        parallelism: 1
        delay: 30s
        failure_action: rollback
        monitor: 60s
        max_failure_ratio: 0.1
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      resources:
        limits:
          cpus: '1.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
    environment:
      - DATABASE_URL_FILE=/run/secrets/database_url
      - JWT_SECRET_FILE=/run/secrets/jwt_secret
      - REDIS_PASSWORD_FILE=/run/secrets/redis_password
      - RUST_LOG=info
      - WORKERS=4
    secrets:
      - database_url
      - jwt_secret
      - redis_password
    configs:
      - source: app_config
        target: /app/config/production.toml
    networks:
      - automation_network
    ports:
      - target: 3000
        published: 3000
        protocol: tcp
        mode: ingress
    volumes:
      - type: bind
        source: /var/run/docker.sock
        target: /var/run/docker.sock
        read_only: true
      - type: volume
        source: app_data
        target: /app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  postgres:
    image: postgres:15-alpine
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.postgres == primary
      restart_policy:
        condition: on-failure
        delay: 10s
        max_attempts: 5
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
    environment:
      - POSTGRES_DB=automation_nation
      - POSTGRES_USER=automation_user
      - POSTGRES_PASSWORD_FILE=/run/secrets/postgres_password
      - POSTGRES_INITDB_ARGS=--auth-host=md5
    secrets:
      - postgres_password
    volumes:
      - type: volume
        source: postgres_data
        target: /var/lib/postgresql/data
      - type: bind
        source: ./postgres/postgresql.conf
        target: /etc/postgresql/postgresql.conf
        read_only: true
    networks:
      - automation_network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U automation_user -d automation_nation"]
      interval: 30s
      timeout: 10s
      retries: 5

  postgres-replica:
    image: postgres:15-alpine
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.postgres == replica
    environment:
      - PGUSER=replication_user
      - POSTGRES_PASSWORD_FILE=/run/secrets/postgres_password
      - POSTGRES_MASTER_SERVICE=postgres
    secrets:
      - postgres_password
    volumes:
      - type: volume
        source: postgres_replica_data
        target: /var/lib/postgresql/data
    networks:
      - automation_network
    command: |
      bash -c "
        pg_basebackup -h postgres -D /var/lib/postgresql/data -U replication_user -v -P -W
        echo 'standby_mode = on' >> /var/lib/postgresql/data/recovery.conf
        echo 'primary_conninfo = \"host=postgres port=5432 user=replication_user\"' >> /var/lib/postgresql/data/recovery.conf
        postgres
      "

  redis:
    image: redis:7-alpine
    deploy:
      replicas: 3
      placement:
        max_replicas_per_node: 1
      restart_policy:
        condition: on-failure
    command: redis-server --appendonly yes --requirepass-file /run/secrets/redis_password
    secrets:
      - redis_password
    volumes:
      - type: volume
        source: redis_data
        target: /data
    networks:
      - automation_network
    healthcheck:
      test: ["CMD", "redis-cli", "--no-auth-warning", "-a", "$(cat /run/secrets/redis_password)", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  nginx:
    image: nginx:alpine
    deploy:
      replicas: 2
      placement:
        constraints:
          - node.role == manager
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
    configs:
      - source: nginx_config
        target: /etc/nginx/nginx.conf
    ports:
      - target: 80
        published: 80
        protocol: tcp
        mode: ingress
      - target: 443
        published: 443
        protocol: tcp
        mode: ingress
    networks:
      - automation_network
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  automation_network:
    driver: overlay
    driver_opts:
      encrypted: "true"
    attachable: false

volumes:
  postgres_data:
    driver: local
    driver_opts:
      type: nfs
      o: addr=nfs-server.local,nolock,soft,rw
      device: :/mnt/postgres_data
  
  postgres_replica_data:
    driver: local
    driver_opts:
      type: nfs
      o: addr=nfs-server.local,nolock,soft,rw
      device: :/mnt/postgres_replica_data
  
  redis_data:
    driver: local
    driver_opts:
      type: nfs
      o: addr=nfs-server.local,nolock,soft,rw
      device: :/mnt/redis_data
  
  app_data:
    driver: local
    driver_opts:
      type: nfs
      o: addr=nfs-server.local,nolock,soft,rw
      device: :/mnt/app_data

secrets:
  database_url:
    external: true
  jwt_secret:
    external: true
  postgres_password:
    external: true
  redis_password:
    external: true

configs:
  app_config:
    external: true
  nginx_config:
    external: true
```

**Load Balancer Configuration (Nginx):**
```nginx
# nginx.conf
events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;
    
    # Logging
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                   '$status $body_bytes_sent "$http_referer" '
                   '"$http_user_agent" "$http_x_forwarded_for"';
    
    access_log /var/log/nginx/access.log main;
    error_log /var/log/nginx/error.log warn;
    
    # Performance optimizations
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 100M;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml
        image/svg+xml;
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;
    limit_req_zone $binary_remote_addr zone=auth:10m rate=10r/m;
    
    # Upstream configuration
    upstream automation_nation_backend {
        least_conn;
        server automation-nation:3000 max_fails=3 fail_timeout=30s;
        keepalive 32;
    }
    
    # Health check endpoint
    server {
        listen 80;
        server_name _;
        
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
    
    # Main application server
    server {
        listen 80;
        listen 443 ssl http2;
        server_name automation-nation.local;
        
        # SSL configuration
        ssl_certificate /etc/ssl/certs/automation-nation.crt;
        ssl_certificate_key /etc/ssl/private/automation-nation.key;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;
        
        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
        add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'";
        
        # API endpoints with rate limiting
        location /api/auth/ {
            limit_req zone=auth burst=20 nodelay;
            proxy_pass http://automation_nation_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 5s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
        
        location /api/ {
            limit_req zone=api burst=200 nodelay;
            proxy_pass http://automation_nation_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 5s;
            proxy_send_timeout 300s;
            proxy_read_timeout 300s;
            
            # WebSocket support for real-time features
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
        
        # Static file serving with caching
        location /static/ {
            proxy_pass http://automation_nation_backend;
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
        
        # Metrics endpoint (restricted access)
        location /metrics {
            allow 172.16.0.0/16;  # Internal networks only
            deny all;
            proxy_pass http://automation_nation_backend;
        }
        
        # Default proxy
        location / {
            proxy_pass http://automation_nation_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

This comprehensive documentation provides detailed coverage of the development processes, testing strategies, build automation, and enterprise deployment procedures for Automation Nation. The documentation bridges the gap between the sophisticated implementation and operational requirements for production deployment.