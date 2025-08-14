# Database Configuration Documentation

## PostgreSQL Migration Guide

This document describes the migration from SQLite to PostgreSQL for the Automation Nation platform.

### Prerequisites

1. **PostgreSQL Server**: Version 12 or higher
2. **Database Credentials**: User with CREATE DATABASE privileges
3. **Network Access**: Connection to PostgreSQL server

### Configuration

#### Environment Variables

Set the following environment variables for PostgreSQL connection:

```bash
# Primary database connection
DATABASE_URL=postgresql://username:password@host:port/database_name

# Example for local development
DATABASE_URL=postgresql://automation_user:automation_password@localhost:5432/automation_nation

# Example for production
DATABASE_URL=postgresql://automation_user:secure_password@postgres.company.com:5432/automation_nation
```

#### Docker Compose Configuration

The platform includes PostgreSQL in docker-compose.yml:

```yaml
postgres:
  image: postgres:15-alpine
  environment:
    POSTGRES_DB: automation_nation
    POSTGRES_USER: automation_user
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
```

### Migration Process

#### 1. Automatic Migration

The application automatically runs migrations on startup:

```rust
let db_manager = DatabaseManager::new().await?;
// Migrations run automatically during initialization
```

#### 2. Manual Migration

To run migrations manually:

```bash
# Using sqlx CLI
cargo install sqlx-cli
sqlx migrate run --database-url $DATABASE_URL

# Using Docker
docker-compose exec automation-nation-web sqlx migrate run
```

### Database Schema

#### Core Tables

1. **users** - User accounts and authentication
2. **roles** - Role definitions and permissions
3. **user_roles** - User-role assignments
4. **sessions** - User login sessions
5. **api_keys** - API key management
6. **system_profiles** - System profiling data
7. **deployment_profiles** - Container deployment profiles
8. **deployments** - Active container deployments
9. **audit_log** - Security audit trail

#### Key Features

- **UUID Primary Keys**: All tables use UUID for better security and distribution
- **JSONB Support**: Flexible metadata and configuration storage
- **Audit Trail**: Comprehensive logging of all security events
- **Indexing**: Optimized indexes for performance
- **Foreign Keys**: Data integrity enforcement

### Performance Tuning

#### Connection Pooling

Default connection pool settings:

```rust
// Pool configuration
max_connections: 10
idle_timeout: 600 seconds
max_lifetime: 1800 seconds
```

#### Recommended PostgreSQL Settings

```postgresql
# postgresql.conf
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
```

### Backup and Recovery

#### Automated Backups

```bash
# Daily backup script
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d).sql

# Compressed backup
pg_dump $DATABASE_URL | gzip > backup_$(date +%Y%m%d).sql.gz
```

#### Point-in-Time Recovery

```bash
# Enable WAL archiving in postgresql.conf
archive_mode = on
archive_command = 'cp %p /backup/archive/%f'

# Create base backup
pg_basebackup -D /backup/base -Ft -z
```

### Monitoring

#### Health Checks

The application provides database health monitoring:

```rust
let stats = db_manager.get_connection_stats().await?;
println!("Active connections: {}", stats.active_connections);
```

#### Key Metrics

- Connection pool utilization
- Query execution time
- Database size and growth
- Index usage statistics
- Lock contention

### Security

#### Best Practices

1. **Encryption**: Use TLS for database connections
2. **Credentials**: Store passwords in environment variables
3. **Access Control**: Limit database user privileges
4. **Auditing**: Enable PostgreSQL audit logging
5. **Backups**: Encrypt backup files

#### Connection Security

```bash
# Use SSL connections
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require

# Verify server certificate
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=verify-full
```

### Troubleshooting

#### Common Issues

1. **Connection Refused**
   - Check PostgreSQL is running
   - Verify network connectivity
   - Check firewall settings

2. **Authentication Failed**
   - Verify username/password
   - Check pg_hba.conf settings
   - Ensure user exists

3. **Migration Errors**
   - Check database permissions
   - Verify schema compatibility
   - Review migration logs

#### Debug Commands

```bash
# Test connection
psql $DATABASE_URL -c "SELECT version();"

# Check active connections
psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity;"

# View migration status
sqlx migrate info --database-url $DATABASE_URL
```

### Development

#### Testing with PostgreSQL

```rust
#[cfg(test)]
async fn test_with_database() {
    let db = create_test_database().await.unwrap();
    // Test code here
}
```

#### Schema Changes

When modifying the database schema:

1. Create new migration file in `migrations/`
2. Use sequential numbering: `YYYYMMDD_NNN_description.sql`
3. Test migration on development database
4. Document breaking changes

### Production Deployment

#### Checklist

- [ ] PostgreSQL server configured and running
- [ ] Database user created with appropriate permissions
- [ ] SSL/TLS encryption enabled
- [ ] Backup strategy implemented
- [ ] Monitoring configured
- [ ] Connection pool tuned for load
- [ ] Environment variables set
- [ ] Migrations tested

#### Rolling Updates

For zero-downtime deployments:

1. Run new migrations (should be backward compatible)
2. Deploy new application version
3. Verify functionality
4. Remove old application instances