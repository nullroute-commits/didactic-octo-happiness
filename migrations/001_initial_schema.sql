-- Database schema for Automation Nation
-- PostgreSQL migration from SQLite

-- Users table for authentication and user management
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    roles TEXT[] NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'active' 
        CHECK (status IN ('active', 'disabled', 'locked', 'pending', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    
    -- Indexes
    CONSTRAINT users_username_length CHECK (char_length(username) >= 3),
    CONSTRAINT users_email_format CHECK (email ~* '^[A-Za-z0-9._%-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Sessions table for user session management
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(512) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    client_ip INET NOT NULL,
    user_agent TEXT,
    metadata JSONB DEFAULT '{}'
);

-- API Keys table for service-to-service authentication
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permissions TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_used TIMESTAMPTZ,
    usage_count BIGINT NOT NULL DEFAULT 0,
    rate_limit JSONB,
    
    CONSTRAINT api_keys_name_length CHECK (char_length(name) >= 1)
);

-- Audit log table for security compliance and monitoring
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    details JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    client_ip INET NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    
    -- At least one of user_id or api_key_id should be set for most actions
    CONSTRAINT audit_log_action_length CHECK (char_length(action) >= 1)
);

-- System profiles table for hardware/software detection results
CREATE TABLE IF NOT EXISTS system_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    os_name VARCHAR(100) NOT NULL,
    os_version VARCHAR(100) NOT NULL,
    kernel_version VARCHAR(100) NOT NULL,
    architecture VARCHAR(50) NOT NULL,
    cpu_model VARCHAR(255) NOT NULL,
    cpu_cores INTEGER NOT NULL CHECK (cpu_cores > 0),
    memory_total_mb BIGINT NOT NULL CHECK (memory_total_mb > 0),
    memory_available_mb BIGINT NOT NULL CHECK (memory_available_mb > 0),
    virtualization_type VARCHAR(100),
    container_runtimes TEXT[] NOT NULL DEFAULT '{}',
    hardware_capabilities JSONB DEFAULT '{}',
    network_interfaces JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Deployment profiles table for application deployment templates
CREATE TABLE IF NOT EXISTS deployment_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    software_name VARCHAR(100) NOT NULL,
    repository JSONB NOT NULL, -- GitHub repository info
    system_requirements JSONB NOT NULL,
    container_config JSONB NOT NULL,
    environment_variables JSONB DEFAULT '{}',
    volumes JSONB DEFAULT '[]',
    ports JSONB DEFAULT '[]',
    health_check JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT deployment_profiles_name_length CHECK (char_length(name) >= 1)
);

-- Deployment instances table for tracking active deployments
CREATE TABLE IF NOT EXISTS deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    profile_id UUID NOT NULL REFERENCES deployment_profiles(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'creating'
        CHECK (status IN ('creating', 'running', 'stopped', 'failed', 'removing')),
    container_id VARCHAR(255),
    container_runtime VARCHAR(50) NOT NULL,
    ports JSONB DEFAULT '[]',
    environment JSONB DEFAULT '{}',
    volumes JSONB DEFAULT '[]',
    health_status VARCHAR(50) DEFAULT 'unknown'
        CHECK (health_status IN ('healthy', 'unhealthy', 'unknown')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT deployments_name_length CHECK (char_length(name) >= 1),
    CONSTRAINT deployments_runtime_valid CHECK (container_runtime IN ('podman', 'docker', 'lxc'))
);

-- OAuth providers table for SSO configuration
CREATE TABLE IF NOT EXISTS oauth_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    provider_type VARCHAR(50) NOT NULL 
        CHECK (provider_type IN ('oidc', 'oauth2', 'saml')),
    client_id VARCHAR(255) NOT NULL,
    client_secret VARCHAR(255) NOT NULL,
    authorization_url VARCHAR(500) NOT NULL,
    token_url VARCHAR(500) NOT NULL,
    userinfo_url VARCHAR(500),
    scopes TEXT[] DEFAULT '{"openid", "profile", "email"}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT oauth_providers_name_length CHECK (char_length(name) >= 1)
);

-- LDAP configuration table for LDAP authentication
CREATE TABLE IF NOT EXISTS ldap_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    server_url VARCHAR(500) NOT NULL,
    bind_dn VARCHAR(255),
    bind_password VARCHAR(255),
    user_search_base VARCHAR(255) NOT NULL,
    user_search_filter VARCHAR(255) NOT NULL DEFAULT '(uid={username})',
    group_search_base VARCHAR(255),
    group_search_filter VARCHAR(255) DEFAULT '(member={user_dn})',
    attribute_mapping JSONB NOT NULL DEFAULT '{"username": "uid", "email": "mail", "display_name": "cn"}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT ldap_config_name_length CHECK (char_length(name) >= 1)
);

-- Password reset tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_keys_owner_id ON api_keys(owner_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
CREATE INDEX IF NOT EXISTS idx_deployments_profile_id ON deployments(profile_id);
CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployments(status);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token ON password_reset_tokens(token);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at columns
CREATE TRIGGER update_system_profiles_updated_at 
    BEFORE UPDATE ON system_profiles 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_deployment_profiles_updated_at 
    BEFORE UPDATE ON deployment_profiles 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_deployments_updated_at 
    BEFORE UPDATE ON deployments 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_oauth_providers_updated_at 
    BEFORE UPDATE ON oauth_providers 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ldap_config_updated_at 
    BEFORE UPDATE ON ldap_config 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();