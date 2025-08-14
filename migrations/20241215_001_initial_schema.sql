-- Initial database schema for Automation Nation
-- This migration creates the core tables for RBAC, deployments, and system profiles

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table for authentication and authorization
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Roles table for RBAC
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- User roles junction table
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_name VARCHAR(255) NOT NULL,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_name)
);

-- Sessions table for user authentication
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(512) UNIQUE NOT NULL,
    client_ip TEXT,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity TIMESTAMP WITH TIME ZONE
);

-- API keys table for programmatic access
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) UNIQUE NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used TIMESTAMP WITH TIME ZONE
);

-- System profiles table
CREATE TABLE system_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    architecture VARCHAR(100) NOT NULL,
    os_name VARCHAR(255) NOT NULL,
    os_version VARCHAR(255),
    kernel_version VARCHAR(255),
    cpu_model TEXT,
    cpu_cores INTEGER,
    memory_total_kb BIGINT,
    memory_available_kb BIGINT,
    disk_info JSONB DEFAULT '[]'::jsonb,
    network_interfaces JSONB DEFAULT '[]'::jsonb,
    installed_packages JSONB DEFAULT '[]'::jsonb,
    running_processes JSONB DEFAULT '[]'::jsonb,
    container_runtime VARCHAR(100),
    container_version VARCHAR(255),
    raw_data JSONB NOT NULL
);

-- Deployment profiles table
CREATE TABLE deployment_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    base_image VARCHAR(255) NOT NULL,
    runtime_type VARCHAR(50) NOT NULL,
    resource_limits JSONB DEFAULT '{}'::jsonb,
    environment_vars JSONB DEFAULT '{}'::jsonb,
    port_mappings JSONB DEFAULT '[]'::jsonb,
    volume_mounts JSONB DEFAULT '[]'::jsonb,
    security_context JSONB DEFAULT '{}'::jsonb,
    health_check JSONB,
    restart_policy VARCHAR(50) DEFAULT 'unless-stopped',
    network_mode VARCHAR(100) DEFAULT 'bridge',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Deployments table for active container instances
CREATE TABLE deployments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    profile_id UUID NOT NULL REFERENCES deployment_profiles(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    container_id VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    runtime_type VARCHAR(50) NOT NULL,
    resource_usage JSONB DEFAULT '{}'::jsonb,
    logs_tail TEXT,
    health_status VARCHAR(50),
    custom_config JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deployed_by UUID REFERENCES users(id)
);

-- Password reset tokens table
CREATE TABLE password_reset_tokens (
    token_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    attempts INTEGER NOT NULL DEFAULT 0,
    ip_address TEXT
);

-- Audit log table for security tracking
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    client_ip TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_system_profiles_created_at ON system_profiles(created_at);
CREATE INDEX idx_system_profiles_architecture ON system_profiles(architecture);
CREATE INDEX idx_deployment_profiles_runtime_type ON deployment_profiles(runtime_type);
CREATE INDEX idx_deployments_profile_id ON deployments(profile_id);
CREATE INDEX idx_deployments_status ON deployments(status);
CREATE INDEX idx_deployments_runtime_type ON deployments(runtime_type);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at);
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);

-- Insert default roles
INSERT INTO roles (name, description, permissions) VALUES 
    ('admin', 'System administrator with full access', '["*"]'::jsonb),
    ('developer', 'Developer with deployment and management permissions', 
     '["system.read", "profiles.read", "profiles.create", "profiles.update", "deployments.read", "deployments.create", "deployments.update", "deployments.restart"]'::jsonb),
    ('viewer', 'Read-only access to system information', 
     '["system.read", "profiles.read", "deployments.read"]'::jsonb);

-- Insert default admin user (password: admin123)
INSERT INTO users (username, email, display_name, password_hash, status) VALUES 
    ('admin', 'admin@automation-nation.local', 'System Administrator', 
     '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewLVkS0/z1.hG/J.', 'active');

-- Assign admin role to default admin user
INSERT INTO user_roles (user_id, role_name) 
SELECT id, 'admin' FROM users WHERE username = 'admin';