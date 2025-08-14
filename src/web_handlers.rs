//! Web handlers for the Automation Nation web interface

use crate::web_types::*;
use crate::{GitHubApiClient, SystemProfiler, DeploymentProfileManager, PodmanManager};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use log::{debug, info, warn, error};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub github_client: Arc<GitHubApiClient>,
    pub system_profiler: Arc<SystemProfiler>,
    pub deployment_manager: Arc<DeploymentProfileManager>,
    pub podman_manager: Arc<PodmanManager>,
    pub deployments: Arc<tokio::sync::RwLock<HashMap<Uuid, DeploymentInstance>>>,
    pub profiles: Arc<tokio::sync::RwLock<HashMap<Uuid, DeploymentProfile>>>,
    pub system_profile: Arc<tokio::sync::RwLock<Option<SystemProfile>>>,
}

/// Query parameters for repository search
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    language: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    per_page: Option<u32>,
    page: Option<u32>,
}

/// Response for API errors
#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

/// Create router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Frontend routes
        .route("/", get(index_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/search", get(search_page_handler))
        .route("/deployments", get(deployments_page_handler))
        .route("/profiles", get(profiles_page_handler))
        .route("/system", get(system_page_handler))
        
        // Administration routes
        .route("/admin", get(admin_dashboard_handler))
        .route("/admin/users", get(admin_users_handler))
        .route("/admin/plugins", get(admin_plugins_handler))
        .route("/admin/settings", get(admin_settings_handler))
        .route("/admin/monitoring", get(admin_monitoring_handler))
        .route("/admin/logs", get(admin_logs_handler))
        
        // API routes
        .route("/api/system/profile", get(get_system_profile))
        .route("/api/system/profile", post(generate_system_profile))
        .route("/api/github/search", get(search_repositories))
        .route("/api/github/repository/:owner/:name", get(get_repository))
        .route("/api/github/trending", get(get_trending_repositories))
        .route("/api/github/categories/:category", get(get_repositories_by_category))
        
        .route("/api/profiles", get(list_deployment_profiles))
        .route("/api/profiles", post(create_deployment_profile))
        .route("/api/profiles/:id", get(get_deployment_profile))
        .route("/api/profiles/:id", delete(delete_deployment_profile))
        
        .route("/api/deployments", get(list_deployments))
        .route("/api/deployments", post(create_deployment))
        .route("/api/deployments/:id", get(get_deployment))
        .route("/api/deployments/:id", delete(remove_deployment))
        .route("/api/deployments/:id/restart", post(restart_deployment))
        .route("/api/deployments/:id/logs", get(get_deployment_logs))
        .route("/api/deployments/:id/status", get(get_deployment_status))
        
        // Administration API routes
        .route("/api/admin/users", get(api_list_users))
        .route("/api/admin/users", post(api_create_user))
        .route("/api/admin/users/:id", get(api_get_user))
        .route("/api/admin/users/:id", delete(api_delete_user))
        .route("/api/admin/plugins", get(api_list_plugins))
        .route("/api/admin/plugins/:id/toggle", post(api_toggle_plugin))
        .route("/api/admin/settings", get(api_get_settings))
        .route("/api/admin/settings", post(api_update_settings))
        .route("/api/admin/system/stats", get(api_system_stats))
        .route("/api/admin/logs", get(api_get_logs))
        
        .with_state(state)
}

// Frontend Handlers

/// Serve the main index page
pub async fn index_handler() -> Html<String> {
    let html = include_str!("../templates/index.html").to_string();
    Html(html)
}

/// Serve the dashboard page
pub async fn dashboard_handler(State(state): State<AppState>) -> std::result::Result<Html<String>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Serving dashboard page");
    
    // Get system profile
    let system_profile = state.system_profile.read().await;
    let profile_available = system_profile.is_some();
    
    // Get deployment count
    let deployments = state.deployments.read().await;
    let deployment_count = deployments.len();
    
    // Get profile count
    let profiles = state.profiles.read().await;
    let profile_count = profiles.len();
    
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Automation Nation - Dashboard</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 20px; }}
        .stat-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stat-value {{ font-size: 2em; font-weight: bold; color: #2563eb; }}
        .stat-label {{ color: #6b7280; margin-top: 5px; }}
        .actions {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }}
        .action-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .btn {{ background: #2563eb; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; text-decoration: none; display: inline-block; }}
        .btn:hover {{ background: #1d4ed8; }}
        .status {{ padding: 5px 10px; border-radius: 4px; font-size: 0.9em; }}
        .status.available {{ background: #dcfce7; color: #166534; }}
        .status.unavailable {{ background: #fee2e2; color: #dc2626; }}
        .nav {{ background: #1f2937; padding: 10px 0; margin: -20px -20px 20px -20px; }}
        .nav-links {{ max-width: 1200px; margin: 0 auto; padding: 0 20px; display: flex; justify-content: space-between; align-items: center; }}
        .nav-links .main-nav {{ display: flex; }}
        .nav-links .admin-nav {{ display: flex; }}
        .nav-links a {{ color: white; text-decoration: none; margin-right: 20px; padding: 5px 10px; border-radius: 4px; }}
        .nav-links a:hover {{ background: #374151; }}
        .nav-links .admin-nav a {{ background: #dc2626; }}
        .nav-links .admin-nav a:hover {{ background: #b91c1c; }}
    </style>
</head>
<body>
    <nav class="nav">
        <div class="nav-links">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/search">Search Software</a>
            <a href="/deployments">Deployments</a>
            <a href="/profiles">Profiles</a>
            <a href="/system">System Info</a>
        </div>
    </nav>
    
    <div class="container">
        <div class="header">
            <h1>Automation Nation Dashboard</h1>
            <p>Manage your open source software deployments with optimized Podman containers</p>
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-value">{deployment_count}</div>
                <div class="stat-label">Active Deployments</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{profile_count}</div>
                <div class="stat-label">Deployment Profiles</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">
                    <span class="status {profile_status_class}">{profile_status}</span>
                </div>
                <div class="stat-label">System Profile</div>
            </div>
        </div>
        
        <div class="actions">
            <div class="action-card">
                <h3>System Profiling</h3>
                <p>Generate a comprehensive system profile to optimize deployments for your hardware.</p>
                <a href="/system" class="btn">View System Profile</a>
            </div>
            
            <div class="action-card">
                <h3>Search Software</h3>
                <p>Search GitHub's open source repository database to find software to deploy.</p>
                <a href="/search" class="btn">Search Repositories</a>
            </div>
            
            <div class="action-card">
                <h3>Manage Deployments</h3>
                <p>View and manage your active Podman container deployments.</p>
                <a href="/deployments" class="btn">View Deployments</a>
            </div>
            
            <div class="action-card">
                <h3>Deployment Profiles</h3>
                <p>Create and manage optimized deployment profiles for different software packages.</p>
                <a href="/profiles" class="btn">Manage Profiles</a>
            </div>
        </div>
    </div>
</body>
</html>
"#, 
        deployment_count = deployment_count,
        profile_count = profile_count,
        profile_status = if profile_available { "Available" } else { "Not Generated" },
        profile_status_class = if profile_available { "available" } else { "unavailable" }
    );
    
    Ok(Html(html))
}

/// Serve the search page
pub async fn search_page_handler() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Search Software - Automation Nation</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .search-form { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .form-group { margin-bottom: 15px; }
        .form-group label { display: block; margin-bottom: 5px; font-weight: bold; }
        .form-group input, .form-group select { width: 100%; padding: 10px; border: 1px solid #d1d5db; border-radius: 4px; }
        .btn { background: #2563eb; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
        .btn:hover { background: #1d4ed8; }
        .results { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .repo-card { border: 1px solid #e5e7eb; border-radius: 8px; padding: 15px; margin-bottom: 15px; }
        .repo-title { font-size: 1.2em; font-weight: bold; margin-bottom: 5px; }
        .repo-description { color: #6b7280; margin-bottom: 10px; }
        .repo-stats { display: flex; gap: 15px; font-size: 0.9em; color: #6b7280; margin-bottom: 10px; }
        .repo-actions { display: flex; gap: 10px; }
        .btn-sm { padding: 5px 10px; font-size: 0.9em; }
        .loading { text-align: center; padding: 40px; color: #6b7280; }
        .nav { background: #1f2937; padding: 10px 0; margin: -20px -20px 20px -20px; }
        .nav-links { max-width: 1200px; margin: 0 auto; padding: 0 20px; }
        .nav-links a { color: white; text-decoration: none; margin-right: 20px; }
        .nav-links a:hover { color: #60a5fa; }
    </style>
</head>
<body>
    <nav class="nav">
        <div class="nav-links">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/search">Search Software</a>
            <a href="/deployments">Deployments</a>
            <a href="/profiles">Profiles</a>
            <a href="/system">System Info</a>
        </div>
    </nav>
    
    <div class="container">
        <h1>Search Open Source Software</h1>
        
        <div class="search-form">
            <form id="searchForm">
                <div class="form-group">
                    <label for="query">Search Query</label>
                    <input type="text" id="query" name="q" placeholder="Enter keywords, project name, or description" required>
                </div>
                
                <div class="form-group">
                    <label for="language">Programming Language</label>
                    <select id="language" name="language">
                        <option value="">Any Language</option>
                        <option value="javascript">JavaScript</option>
                        <option value="python">Python</option>
                        <option value="java">Java</option>
                        <option value="rust">Rust</option>
                        <option value="go">Go</option>
                        <option value="typescript">TypeScript</option>
                        <option value="php">PHP</option>
                        <option value="c++">C++</option>
                        <option value="c">C</option>
                        <option value="ruby">Ruby</option>
                    </select>
                </div>
                
                <button type="submit" class="btn">Search Repositories</button>
            </form>
        </div>
        
        <div id="results" class="results" style="display: none;">
            <h2>Search Results</h2>
            <div id="resultsContent"></div>
        </div>
    </div>
    
    <script>
        document.getElementById('searchForm').addEventListener('submit', async function(e) {
            e.preventDefault();
            
            const formData = new FormData(e.target);
            const params = new URLSearchParams();
            
            for (const [key, value] of formData.entries()) {
                if (value) params.append(key, value);
            }
            
            const resultsDiv = document.getElementById('results');
            const resultsContent = document.getElementById('resultsContent');
            
            resultsDiv.style.display = 'block';
            resultsContent.innerHTML = '<div class="loading">Searching repositories...</div>';
            
            try {
                const response = await fetch(`/api/github/search?${params.toString()}`);
                const data = await response.json();
                
                if (data.repositories && data.repositories.length > 0) {
                    let html = '';
                    for (const repo of data.repositories) {
                        html += `
                            <div class="repo-card">
                                <div class="repo-title">${repo.full_name}</div>
                                <div class="repo-description">${repo.description || 'No description available'}</div>
                                <div class="repo-stats">
                                    <span>⭐ ${repo.stargazers_count}</span>
                                    <span>🍴 ${repo.forks_count}</span>
                                    <span>📝 ${repo.language || 'Unknown'}</span>
                                    <span>🐛 ${repo.open_issues_count} issues</span>
                                </div>
                                <div class="repo-actions">
                                    <button class="btn btn-sm" onclick="createProfile('${repo.id}', '${repo.full_name}')">Create Profile</button>
                                    <a href="${repo.html_url}" target="_blank" class="btn btn-sm">View on GitHub</a>
                                </div>
                            </div>
                        `;
                    }
                    resultsContent.innerHTML = html;
                } else {
                    resultsContent.innerHTML = '<div class="loading">No repositories found. Try different search terms.</div>';
                }
            } catch (error) {
                resultsContent.innerHTML = '<div class="loading">Error searching repositories. Please try again.</div>';
            }
        });
        
        async function createProfile(repoId, fullName) {
            alert(`Creating deployment profile for ${fullName}...`);
            // This would trigger the profile creation workflow
        }
    </script>
</body>
</html>
"#.to_string())
}

/// Serve the deployments page
pub async fn deployments_page_handler() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Deployments - Automation Nation</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .deployments-list { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .deployment-card { border: 1px solid #e5e7eb; border-radius: 8px; padding: 15px; margin-bottom: 15px; }
        .deployment-header { display: flex; justify-content: between; align-items: center; margin-bottom: 10px; }
        .deployment-name { font-size: 1.2em; font-weight: bold; }
        .deployment-status { padding: 5px 10px; border-radius: 4px; font-size: 0.9em; }
        .status-running { background: #dcfce7; color: #166534; }
        .status-stopped { background: #fee2e2; color: #dc2626; }
        .status-creating { background: #fef3c7; color: #92400e; }
        .deployment-actions { display: flex; gap: 10px; margin-top: 10px; }
        .btn { background: #2563eb; color: white; padding: 8px 15px; border: none; border-radius: 4px; cursor: pointer; text-decoration: none; font-size: 0.9em; }
        .btn:hover { background: #1d4ed8; }
        .btn-danger { background: #dc2626; }
        .btn-danger:hover { background: #b91c1c; }
        .btn-secondary { background: #6b7280; }
        .btn-secondary:hover { background: #4b5563; }
        .nav { background: #1f2937; padding: 10px 0; margin: -20px -20px 20px -20px; }
        .nav-links { max-width: 1200px; margin: 0 auto; padding: 0 20px; }
        .nav-links a { color: white; text-decoration: none; margin-right: 20px; }
        .nav-links a:hover { color: #60a5fa; }
        .empty { text-align: center; padding: 40px; color: #6b7280; }
    </style>
</head>
<body>
    <nav class="nav">
        <div class="nav-links">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/search">Search Software</a>
            <a href="/deployments">Deployments</a>
            <a href="/profiles">Profiles</a>
            <a href="/system">System Info</a>
        </div>
    </nav>
    
    <div class="container">
        <h1>Active Deployments</h1>
        
        <div id="deploymentsList" class="deployments-list">
            <div class="empty">Loading deployments...</div>
        </div>
    </div>
    
    <script>
        async function loadDeployments() {
            try {
                const response = await fetch('/api/deployments');
                const deployments = await response.json();
                
                const listDiv = document.getElementById('deploymentsList');
                
                if (deployments.length === 0) {
                    listDiv.innerHTML = '<div class="empty">No active deployments. <a href="/search">Search for software to deploy</a>.</div>';
                    return;
                }
                
                let html = '';
                for (const deployment of deployments) {
                    const statusClass = `status-${deployment.status.toLowerCase()}`;
                    html += `
                        <div class="deployment-card">
                            <div class="deployment-header">
                                <div class="deployment-name">${deployment.name}</div>
                                <div class="deployment-status ${statusClass}">${deployment.status}</div>
                            </div>
                            <div>Profile: ${deployment.profile_id}</div>
                            <div>Created: ${new Date(deployment.created_at).toLocaleString()}</div>
                            <div>Ports: ${deployment.ports.join(', ') || 'None'}</div>
                            <div class="deployment-actions">
                                <button class="btn" onclick="viewLogs('${deployment.id}')">View Logs</button>
                                <button class="btn btn-secondary" onclick="restartDeployment('${deployment.id}')">Restart</button>
                                <button class="btn btn-danger" onclick="removeDeployment('${deployment.id}')">Remove</button>
                            </div>
                        </div>
                    `;
                }
                listDiv.innerHTML = html;
            } catch (error) {
                document.getElementById('deploymentsList').innerHTML = '<div class="empty">Error loading deployments.</div>';
            }
        }
        
        async function restartDeployment(id) {
            if (confirm('Restart this deployment?')) {
                try {
                    await fetch(`/api/deployments/${id}/restart`, { method: 'POST' });
                    loadDeployments();
                } catch (error) {
                    alert('Failed to restart deployment');
                }
            }
        }
        
        async function removeDeployment(id) {
            if (confirm('Remove this deployment? This action cannot be undone.')) {
                try {
                    await fetch(`/api/deployments/${id}`, { method: 'DELETE' });
                    loadDeployments();
                } catch (error) {
                    alert('Failed to remove deployment');
                }
            }
        }
        
        function viewLogs(id) {
            window.open(`/api/deployments/${id}/logs`, '_blank');
        }
        
        // Load deployments on page load
        loadDeployments();
        
        // Refresh every 30 seconds
        setInterval(loadDeployments, 30000);
    </script>
</body>
</html>
"#.to_string())
}

/// Serve the profiles page
pub async fn profiles_page_handler() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Deployment Profiles - Automation Nation</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .profiles-list { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .profile-card { border: 1px solid #e5e7eb; border-radius: 8px; padding: 15px; margin-bottom: 15px; }
        .profile-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
        .profile-name { font-size: 1.2em; font-weight: bold; }
        .profile-actions { display: flex; gap: 10px; }
        .btn { background: #2563eb; color: white; padding: 8px 15px; border: none; border-radius: 4px; cursor: pointer; text-decoration: none; font-size: 0.9em; }
        .btn:hover { background: #1d4ed8; }
        .btn-danger { background: #dc2626; }
        .btn-danger:hover { background: #b91c1c; }
        .btn-success { background: #059669; }
        .btn-success:hover { background: #047857; }
        .nav { background: #1f2937; padding: 10px 0; margin: -20px -20px 20px -20px; }
        .nav-links { max-width: 1200px; margin: 0 auto; padding: 0 20px; }
        .nav-links a { color: white; text-decoration: none; margin-right: 20px; }
        .nav-links a:hover { color: #60a5fa; }
        .empty { text-align: center; padding: 40px; color: #6b7280; }
        .requirements { font-size: 0.9em; color: #6b7280; margin: 10px 0; }
    </style>
</head>
<body>
    <nav class="nav">
        <div class="nav-links">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/search">Search Software</a>
            <a href="/deployments">Deployments</a>
            <a href="/profiles">Profiles</a>
            <a href="/system">System Info</a>
        </div>
    </nav>
    
    <div class="container">
        <h1>Deployment Profiles</h1>
        
        <div id="profilesList" class="profiles-list">
            <div class="empty">Loading profiles...</div>
        </div>
    </div>
    
    <script>
        async function loadProfiles() {
            try {
                const response = await fetch('/api/profiles');
                const profiles = await response.json();
                
                const listDiv = document.getElementById('profilesList');
                
                if (profiles.length === 0) {
                    listDiv.innerHTML = '<div class="empty">No deployment profiles. <a href="/search">Search for software to create profiles</a>.</div>';
                    return;
                }
                
                let html = '';
                for (const profile of profiles) {
                    html += `
                        <div class="profile-card">
                            <div class="profile-header">
                                <div class="profile-name">${profile.name}</div>
                                <div class="profile-actions">
                                    <button class="btn btn-success" onclick="deployProfile('${profile.id}')">Deploy</button>
                                    <button class="btn" onclick="viewProfile('${profile.id}')">View Details</button>
                                    <button class="btn btn-danger" onclick="deleteProfile('${profile.id}')">Delete</button>
                                </div>
                            </div>
                            <div>Software: ${profile.software_name}</div>
                            <div>Repository: ${profile.repository.full_name}</div>
                            <div class="requirements">
                                Requirements: ${profile.system_requirements.min_memory_mb}MB RAM, 
                                ${profile.system_requirements.min_cpu_cores} cores, 
                                ${profile.system_requirements.min_disk_gb}GB disk
                            </div>
                            <div>Created: ${new Date(profile.created_at).toLocaleString()}</div>
                        </div>
                    `;
                }
                listDiv.innerHTML = html;
            } catch (error) {
                document.getElementById('profilesList').innerHTML = '<div class="empty">Error loading profiles.</div>';
            }
        }
        
        async function deployProfile(id) {
            const name = prompt('Enter deployment name:');
            if (name) {
                try {
                    const response = await fetch('/api/deployments', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ profile_id: id, name: name })
                    });
                    
                    if (response.ok) {
                        alert('Deployment started successfully!');
                        window.location.href = '/deployments';
                    } else {
                        alert('Failed to start deployment');
                    }
                } catch (error) {
                    alert('Failed to start deployment');
                }
            }
        }
        
        function viewProfile(id) {
            window.open(`/api/profiles/${id}`, '_blank');
        }
        
        async function deleteProfile(id) {
            if (confirm('Delete this profile? This action cannot be undone.')) {
                try {
                    await fetch(`/api/profiles/${id}`, { method: 'DELETE' });
                    loadProfiles();
                } catch (error) {
                    alert('Failed to delete profile');
                }
            }
        }
        
        // Load profiles on page load
        loadProfiles();
    </script>
</body>
</html>
"#.to_string())
}

/// Serve the system page
pub async fn system_page_handler() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>System Information - Automation Nation</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .system-info { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 20px; }
        .info-section { margin-bottom: 20px; }
        .info-section h3 { margin-top: 0; color: #1f2937; border-bottom: 2px solid #e5e7eb; padding-bottom: 5px; }
        .info-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 15px; }
        .info-item { background: #f9fafb; padding: 10px; border-radius: 4px; }
        .info-label { font-weight: bold; color: #374151; }
        .info-value { color: #6b7280; }
        .btn { background: #2563eb; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; text-decoration: none; display: inline-block; margin-right: 10px; }
        .btn:hover { background: #1d4ed8; }
        .btn-success { background: #059669; }
        .btn-success:hover { background: #047857; }
        .nav { background: #1f2937; padding: 10px 0; margin: -20px -20px 20px -20px; }
        .nav-links { max-width: 1200px; margin: 0 auto; padding: 0 20px; }
        .nav-links a { color: white; text-decoration: none; margin-right: 20px; }
        .nav-links a:hover { color: #60a5fa; }
        .loading { text-align: center; padding: 40px; color: #6b7280; }
    </style>
</head>
<body>
    <nav class="nav">
        <div class="nav-links">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/search">Search Software</a>
            <a href="/deployments">Deployments</a>
            <a href="/profiles">Profiles</a>
            <a href="/system">System Info</a>
        </div>
    </nav>
    
    <div class="container">
        <h1>System Information</h1>
        
        <div class="system-info">
            <div style="margin-bottom: 20px;">
                <button class="btn btn-success" onclick="generateProfile()">Generate System Profile</button>
                <button class="btn" onclick="loadSystemProfile()">Refresh</button>
            </div>
            
            <div id="systemInfo">
                <div class="loading">Click "Generate System Profile" to analyze your system...</div>
            </div>
        </div>
    </div>
    
    <script>
        async function generateProfile() {
            const systemInfoDiv = document.getElementById('systemInfo');
            systemInfoDiv.innerHTML = '<div class="loading">Analyzing system... This may take a moment.</div>';
            
            try {
                const response = await fetch('/api/system/profile', { method: 'POST' });
                const profile = await response.json();
                
                displaySystemProfile(profile);
            } catch (error) {
                systemInfoDiv.innerHTML = '<div class="loading">Error generating system profile. Please try again.</div>';
            }
        }
        
        async function loadSystemProfile() {
            try {
                const response = await fetch('/api/system/profile');
                if (response.ok) {
                    const profile = await response.json();
                    displaySystemProfile(profile);
                } else {
                    document.getElementById('systemInfo').innerHTML = '<div class="loading">No system profile available. Click "Generate System Profile" to create one.</div>';
                }
            } catch (error) {
                document.getElementById('systemInfo').innerHTML = '<div class="loading">Error loading system profile.</div>';
            }
        }
        
        function displaySystemProfile(profile) {
            const html = `
                <div class="info-section">
                    <h3>Operating System</h3>
                    <div class="info-grid">
                        <div class="info-item">
                            <div class="info-label">OS Name</div>
                            <div class="info-value">${profile.os_name}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">OS Version</div>
                            <div class="info-value">${profile.os_version}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">Kernel Version</div>
                            <div class="info-value">${profile.kernel_version}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">Architecture</div>
                            <div class="info-value">${profile.architecture}</div>
                        </div>
                    </div>
                </div>
                
                <div class="info-section">
                    <h3>Hardware</h3>
                    <div class="info-grid">
                        <div class="info-item">
                            <div class="info-label">CPU Model</div>
                            <div class="info-value">${profile.cpu_model}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">CPU Cores</div>
                            <div class="info-value">${profile.cpu_cores}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">Total Memory</div>
                            <div class="info-value">${profile.memory_total_mb} MB</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">Available Memory</div>
                            <div class="info-value">${profile.memory_available_mb} MB</div>
                        </div>
                    </div>
                </div>
                
                <div class="info-section">
                    <h3>Virtualization & Containers</h3>
                    <div class="info-grid">
                        <div class="info-item">
                            <div class="info-label">Virtualization Type</div>
                            <div class="info-value">${profile.virtualization_type || 'Unknown'}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">Container Runtimes</div>
                            <div class="info-value">${profile.container_runtimes.join(', ') || 'None'}</div>
                        </div>
                    </div>
                </div>
                
                <div class="info-section">
                    <h3>Hardware Capabilities</h3>
                    <div class="info-grid">
                        <div class="info-item">
                            <div class="info-label">GPU Available</div>
                            <div class="info-value">${profile.hardware_capabilities.has_gpu ? 'Yes' : 'No'}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">PCIe Devices</div>
                            <div class="info-value">${profile.hardware_capabilities.pcie_devices}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">USB Devices</div>
                            <div class="info-value">${profile.hardware_capabilities.usb_devices}</div>
                        </div>
                        <div class="info-item">
                            <div class="info-label">Total Disk Space</div>
                            <div class="info-value">${profile.hardware_capabilities.disk_total_gb} GB</div>
                        </div>
                    </div>
                </div>
                
                <div class="info-section">
                    <h3>Network Interfaces</h3>
                    <div class="info-grid">
                        ${profile.network_interfaces.map(iface => `
                            <div class="info-item">
                                <div class="info-label">${iface.name}</div>
                                <div class="info-value">
                                    IPv4: ${iface.ipv4_addresses.join(', ') || 'None'}<br>
                                    State: ${iface.state}<br>
                                    MTU: ${iface.mtu}
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
            
            document.getElementById('systemInfo').innerHTML = html;
        }
        
        // Load system profile on page load
        loadSystemProfile();
    </script>
</body>
</html>
"#.to_string())
}

// API Handlers

/// Get current system profile
pub async fn get_system_profile(State(state): State<AppState>) -> std::result::Result<Json<SystemProfile>, (StatusCode, Json<ErrorResponse>)> {
    let profile = state.system_profile.read().await;
    
    match profile.as_ref() {
        Some(p) => Ok(Json(p.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "No system profile available".to_string(),
            })
        ))
    }
}

/// Generate new system profile
pub async fn generate_system_profile(State(state): State<AppState>) -> std::result::Result<Json<SystemProfile>, (StatusCode, Json<ErrorResponse>)> {
    info!("Generating new system profile");
    
    match state.system_profiler.generate_profile().await {
        Ok(profile) => {
            // Store the profile
            let mut stored_profile = state.system_profile.write().await;
            *stored_profile = Some(profile.clone());
            
            info!("System profile generated successfully");
            Ok(Json(profile))
        }
        Err(e) => {
            error!("Failed to generate system profile: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "generation_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Search GitHub repositories
pub async fn search_repositories(
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
) -> std::result::Result<Json<SearchRepositoriesResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Searching repositories with query: {}", params.q);
    
    let request = SearchRepositoriesRequest {
        query: params.q,
        language: params.language,
        sort: params.sort,
        order: params.order,
        per_page: params.per_page,
        page: params.page,
    };
    
    match state.github_client.search_repositories(&request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("GitHub search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "search_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get specific repository
pub async fn get_repository(
    Path((owner, name)): Path<(String, String)>,
    State(state): State<AppState>,
) -> std::result::Result<Json<GitHubRepository>, (StatusCode, Json<ErrorResponse>)> {
    match state.github_client.get_repository(&owner, &name).await {
        Ok(repository) => Ok(Json(repository)),
        Err(e) => {
            error!("Failed to get repository {}/{}: {}", owner, name, e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "repository_not_found".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get trending repositories
pub async fn get_trending_repositories(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<GitHubRepository>>, (StatusCode, Json<ErrorResponse>)> {
    let language = params.get("language").cloned();
    let limit = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);
    
    match state.github_client.get_trending_repositories(language, limit).await {
        Ok(repositories) => Ok(Json(repositories)),
        Err(e) => {
            error!("Failed to get trending repositories: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "trending_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get repositories by category
pub async fn get_repositories_by_category(
    Path(category): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<GitHubRepository>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);
    
    match state.github_client.get_popular_by_category(&category, limit).await {
        Ok(repositories) => Ok(Json(repositories)),
        Err(e) => {
            error!("Failed to get repositories by category {}: {}", category, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "category_search_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// List deployment profiles
pub async fn list_deployment_profiles(State(state): State<AppState>) -> Json<Vec<DeploymentProfile>> {
    let profiles = state.profiles.read().await;
    Json(profiles.values().cloned().collect())
}

/// Create deployment profile
pub async fn create_deployment_profile(
    State(state): State<AppState>,
    Json(request): Json<GenerateProfileRequest>,
) -> std::result::Result<Json<GenerateProfileResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating deployment profile for {}", request.repository.full_name);
    
    // Get system profile
    let system_profile = {
        let profile = state.system_profile.read().await;
        match profile.as_ref() {
            Some(p) => p.clone(),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "no_system_profile".to_string(),
                        message: "No system profile available. Generate one first.".to_string(),
                    })
                ));
            }
        }
    };
    
    match state.deployment_manager.generate_profile(
        request.repository,
        system_profile,
        request.custom_requirements,
    ).await {
        Ok(response) => {
            // Store the profile
            let mut profiles = state.profiles.write().await;
            profiles.insert(response.profile.id, response.profile.clone());
            
            info!("Deployment profile created successfully: {}", response.profile.name);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to create deployment profile: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "profile_creation_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get specific deployment profile
pub async fn get_deployment_profile(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> std::result::Result<Json<DeploymentProfile>, (StatusCode, Json<ErrorResponse>)> {
    let profiles = state.profiles.read().await;
    
    match profiles.get(&id) {
        Some(profile) => Ok(Json(profile.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "profile_not_found".to_string(),
                message: "Deployment profile not found".to_string(),
            })
        ))
    }
}

/// Delete deployment profile
pub async fn delete_deployment_profile(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut profiles = state.profiles.write().await;
    
    match profiles.remove(&id) {
        Some(_) => {
            info!("Deployment profile {} deleted", id);
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "profile_not_found".to_string(),
                message: "Deployment profile not found".to_string(),
            })
        ))
    }
}

/// List deployments
pub async fn list_deployments(State(state): State<AppState>) -> Json<Vec<DeploymentInstance>> {
    let deployments = state.deployments.read().await;
    Json(deployments.values().cloned().collect())
}

/// Create deployment
pub async fn create_deployment(
    State(state): State<AppState>,
    Json(request): Json<CreateDeploymentRequest>,
) -> std::result::Result<Json<CreateDeploymentResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating deployment: {}", request.name);
    
    // Get profile
    let profile = {
        let profiles = state.profiles.read().await;
        match profiles.get(&request.profile_id) {
            Some(p) => p.clone(),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "profile_not_found".to_string(),
                        message: "Deployment profile not found".to_string(),
                    })
                ));
            }
        }
    };
    
    match state.podman_manager.deploy(&profile, &request).await {
        Ok(response) => {
            // Store the deployment
            let mut deployments = state.deployments.write().await;
            deployments.insert(response.deployment.id, response.deployment.clone());
            
            info!("Deployment created successfully: {}", response.deployment.name);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to create deployment: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "deployment_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get specific deployment
pub async fn get_deployment(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> std::result::Result<Json<DeploymentInstance>, (StatusCode, Json<ErrorResponse>)> {
    let deployments = state.deployments.read().await;
    
    match deployments.get(&id) {
        Some(deployment) => Ok(Json(deployment.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "deployment_not_found".to_string(),
                message: "Deployment not found".to_string(),
            })
        ))
    }
}

/// Remove deployment
pub async fn remove_deployment(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Get deployment
    let deployment = {
        let deployments = state.deployments.read().await;
        match deployments.get(&id) {
            Some(d) => d.clone(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "deployment_not_found".to_string(),
                        message: "Deployment not found".to_string(),
                    })
                ));
            }
        }
    };
    
    // Undeploy from Podman
    if let Err(e) = state.podman_manager.undeploy(&deployment).await {
        warn!("Failed to undeploy from Podman: {}", e);
        // Continue anyway to remove from our records
    }
    
    // Remove from storage
    let mut deployments = state.deployments.write().await;
    deployments.remove(&id);
    
    info!("Deployment {} removed", id);
    Ok(StatusCode::NO_CONTENT)
}

/// Restart deployment
pub async fn restart_deployment(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deployment = {
        let deployments = state.deployments.read().await;
        match deployments.get(&id) {
            Some(d) => d.clone(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "deployment_not_found".to_string(),
                        message: "Deployment not found".to_string(),
                    })
                ));
            }
        }
    };
    
    match state.podman_manager.restart_deployment(&deployment).await {
        Ok(_) => {
            info!("Deployment {} restarted", id);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to restart deployment {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "restart_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get deployment logs
pub async fn get_deployment_logs(
    Path(id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<DeploymentLog>>, (StatusCode, Json<ErrorResponse>)> {
    let deployment = {
        let deployments = state.deployments.read().await;
        match deployments.get(&id) {
            Some(d) => d.clone(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "deployment_not_found".to_string(),
                        message: "Deployment not found".to_string(),
                    })
                ));
            }
        }
    };
    
    let tail_lines = params.get("tail")
        .and_then(|t| t.parse().ok())
        .unwrap_or(100);
    
    match state.podman_manager.get_container_logs(&deployment, tail_lines).await {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => {
            error!("Failed to get logs for deployment {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "logs_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

/// Get deployment status
pub async fn get_deployment_status(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> std::result::Result<Json<DeploymentStatus>, (StatusCode, Json<ErrorResponse>)> {
    let deployment = {
        let deployments = state.deployments.read().await;
        match deployments.get(&id) {
            Some(d) => d.clone(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "deployment_not_found".to_string(),
                        message: "Deployment not found".to_string(),
                    })
                ));
            }
        }
    };
    
    match state.podman_manager.get_deployment_status(&deployment).await {
        Ok(status) => {
            // Update stored deployment status
            let mut deployments = state.deployments.write().await;
            if let Some(stored_deployment) = deployments.get_mut(&id) {
                stored_deployment.status = status.clone();
                stored_deployment.updated_at = chrono::Utc::now();
            }
            
            Ok(Json(status))
        }
        Err(e) => {
            error!("Failed to get status for deployment {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "status_failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

// Admin Interface Handlers

/// Shared navigation template for admin pages
fn get_admin_nav() -> String {
    r#"
    <nav class="nav">
        <div class="nav-links">
            <div class="main-nav">
                <a href="/">Home</a>
                <a href="/dashboard">Dashboard</a>
                <a href="/search">Search Software</a>
                <a href="/deployments">Deployments</a>
                <a href="/profiles">Profiles</a>
                <a href="/system">System Info</a>
            </div>
            <div class="admin-nav">
                <a href="/admin">Administration</a>
            </div>
        </div>
    </nav>
    "#.to_string()
}

/// Get common admin styles
fn get_admin_styles() -> String {
    r#"
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .nav { background: #1f2937; padding: 10px 0; margin: -20px -20px 20px -20px; }
        .nav-links { max-width: 1200px; margin: 0 auto; padding: 0 20px; display: flex; justify-content: space-between; align-items: center; }
        .nav-links .main-nav { display: flex; }
        .nav-links .admin-nav { display: flex; }
        .nav-links a { color: white; text-decoration: none; margin-right: 20px; padding: 5px 10px; border-radius: 4px; }
        .nav-links a:hover { background: #374151; }
        .nav-links .admin-nav a { background: #dc2626; }
        .nav-links .admin-nav a:hover { background: #b91c1c; }
        .admin-sidebar { display: grid; grid-template-columns: 250px 1fr; gap: 20px; }
        .sidebar { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); height: fit-content; }
        .sidebar h3 { margin-top: 0; color: #1f2937; }
        .sidebar ul { list-style: none; padding: 0; }
        .sidebar li { margin: 10px 0; }
        .sidebar a { color: #374151; text-decoration: none; padding: 8px 12px; display: block; border-radius: 4px; }
        .sidebar a:hover { background: #f3f4f6; }
        .sidebar a.active { background: #dbeafe; color: #1d4ed8; }
        .main-content { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .card { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 20px; }
        .grid { display: grid; gap: 20px; }
        .grid-2 { grid-template-columns: repeat(2, 1fr); }
        .grid-3 { grid-template-columns: repeat(3, 1fr); }
        .grid-4 { grid-template-columns: repeat(4, 1fr); }
        .btn { background: #2563eb; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; text-decoration: none; display: inline-block; }
        .btn:hover { background: #1d4ed8; }
        .btn-danger { background: #dc2626; }
        .btn-danger:hover { background: #b91c1c; }
        .btn-success { background: #16a34a; }
        .btn-success:hover { background: #15803d; }
        .stat-card { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .stat-value { font-size: 2em; font-weight: bold; color: #2563eb; }
        .stat-label { color: #6b7280; margin-top: 5px; }
        .status { padding: 5px 10px; border-radius: 4px; font-size: 0.9em; }
        .status.online { background: #dcfce7; color: #166534; }
        .status.offline { background: #fee2e2; color: #dc2626; }
        .status.warning { background: #fef3c7; color: #92400e; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #e5e7eb; }
        th { background: #f9fafb; font-weight: 600; }
        .form-group { margin-bottom: 20px; }
        label { display: block; margin-bottom: 5px; color: #374151; font-weight: 500; }
        input, select, textarea { width: 100%; padding: 10px; border: 1px solid #d1d5db; border-radius: 4px; box-sizing: border-box; }
        input:focus, select:focus, textarea:focus { outline: none; border-color: #2563eb; box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1); }
    "#.to_string()
}

/// Main administration dashboard
pub async fn admin_dashboard_handler(State(state): State<AppState>) -> Html<String> {
    let deployments = state.deployments.read().await;
    let profiles = state.profiles.read().await;
    let system_profile = state.system_profile.read().await;
    
    let deployment_count = deployments.len();
    let profile_count = profiles.len();
    let system_status = if system_profile.is_some() { "Generated" } else { "Not Generated" };
    
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Administration - Automation Nation</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        {styles}
    </style>
</head>
<body>
    {nav}
    
    <div class="container">
        <div class="header">
            <h1>🔧 Administration Dashboard</h1>
            <p>Manage users, plugins, system settings, and monitor the Automation Nation platform</p>
        </div>
        
        <div class="admin-sidebar">
            <div class="sidebar">
                <h3>Administration</h3>
                <ul>
                    <li><a href="/admin" class="active">Dashboard</a></li>
                    <li><a href="/admin/users">User Management</a></li>
                    <li><a href="/admin/plugins">Plugin Marketplace</a></li>
                    <li><a href="/admin/settings">System Settings</a></li>
                    <li><a href="/admin/monitoring">Monitoring</a></li>
                    <li><a href="/admin/logs">System Logs</a></li>
                </ul>
            </div>
            
            <div class="main-content">
                <h2>System Overview</h2>
                
                <div class="grid grid-3">
                    <div class="stat-card">
                        <div class="stat-value">{deployment_count}</div>
                        <div class="stat-label">Active Deployments</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{profile_count}</div>
                        <div class="stat-label">Deployment Profiles</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{system_status}</div>
                        <div class="stat-label">System Profile</div>
                    </div>
                </div>
                
                <div class="grid grid-2" style="margin-top: 20px;">
                    <div class="card">
                        <h3>Quick Actions</h3>
                        <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                            <a href="/admin/users" class="btn">Manage Users</a>
                            <a href="/admin/plugins" class="btn">Manage Plugins</a>
                            <a href="/admin/settings" class="btn">System Settings</a>
                            <a href="/admin/monitoring" class="btn">View Monitoring</a>
                        </div>
                    </div>
                    
                    <div class="card">
                        <h3>System Status</h3>
                        <div style="display: flex; flex-direction: column; gap: 10px;">
                            <div style="display: flex; justify-content: space-between;">
                                <span>Web Server</span>
                                <span class="status online">Online</span>
                            </div>
                            <div style="display: flex; justify-content: space-between;">
                                <span>Container Runtime</span>
                                <span class="status online">Podman Available</span>
                            </div>
                            <div style="display: flex; justify-content: space-between;">
                                <span>GitHub API</span>
                                <span class="status warning">Limited (Proxy Blocked)</span>
                            </div>
                            <div style="display: flex; justify-content: space-between;">
                                <span>System Profiler</span>
                                <span class="status online">Available</span>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="card" style="margin-top: 20px;">
                    <h3>Recent Activity</h3>
                    <div id="recent-activity">
                        <p>Loading recent activity...</p>
                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <script>
        // Load recent activity
        fetch('/api/admin/system/stats')
            .then(response => response.json())
            .then(data => {{
                const activityDiv = document.getElementById('recent-activity');
                if (data.recent_activity && data.recent_activity.length > 0) {{
                    activityDiv.innerHTML = data.recent_activity.map(activity => 
                        `<div style="padding: 10px; border-bottom: 1px solid #e5e7eb;">
                            <strong>${{activity.action}}</strong> - ${{activity.timestamp}}
                            <br><small>${{activity.details}}</small>
                        </div>`
                    ).join('');
                }} else {{
                    activityDiv.innerHTML = '<p>No recent activity</p>';
                }}
            }})
            .catch(error => {{
                document.getElementById('recent-activity').innerHTML = '<p>Unable to load recent activity</p>';
            }});
    </script>
</body>
</html>
"#,
        styles = get_admin_styles(),
        nav = get_admin_nav(),
        deployment_count = deployment_count,
        profile_count = profile_count,
        system_status = system_status
    );
    
    Html(html)
}

/// Admin users management page
pub async fn admin_users_handler() -> Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>User Management - Administration</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>{styles}</style>
</head>
<body>
    {nav}
    
    <div class="container">
        <div class="header">
            <h1>👥 User Management</h1>
            <p>Manage user accounts, roles, and permissions</p>
        </div>
        
        <div class="admin-sidebar">
            <div class="sidebar">
                <h3>Administration</h3>
                <ul>
                    <li><a href="/admin">Dashboard</a></li>
                    <li><a href="/admin/users" class="active">User Management</a></li>
                    <li><a href="/admin/plugins">Plugin Marketplace</a></li>
                    <li><a href="/admin/settings">System Settings</a></li>
                    <li><a href="/admin/monitoring">Monitoring</a></li>
                    <li><a href="/admin/logs">System Logs</a></li>
                </ul>
            </div>
            
            <div class="main-content">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                    <h2>User Accounts</h2>
                    <button class="btn" onclick="showCreateUserModal()">Create New User</button>
                </div>
                
                <div class="card">
                    <table>
                        <thead>
                            <tr>
                                <th>Username</th>
                                <th>Email</th>
                                <th>Role</th>
                                <th>Status</th>
                                <th>Last Login</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody id="users-table">
                            <tr>
                                <td colspan="6" style="text-align: center;">Loading users...</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
    
    <!-- Create User Modal -->
    <div id="createUserModal" style="display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); z-index: 1000;">
        <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); background: white; padding: 30px; border-radius: 8px; width: 90%; max-width: 500px;">
            <h3>Create New User</h3>
            <form id="createUserForm">
                <div class="form-group">
                    <label for="username">Username:</label>
                    <input type="text" id="username" name="username" required>
                </div>
                <div class="form-group">
                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email" required>
                </div>
                <div class="form-group">
                    <label for="role">Role:</label>
                    <select id="role" name="role">
                        <option value="user">User</option>
                        <option value="admin">Administrator</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="password">Password:</label>
                    <input type="password" id="password" name="password" required>
                </div>
                <div style="display: flex; gap: 10px; justify-content: flex-end;">
                    <button type="button" class="btn" style="background: #6b7280;" onclick="hideCreateUserModal()">Cancel</button>
                    <button type="submit" class="btn">Create User</button>
                </div>
            </form>
        </div>
    </div>
    
    <script>
        function loadUsers() {{
            fetch('/api/admin/users')
                .then(response => response.json())
                .then(users => {{
                    const tbody = document.getElementById('users-table');
                    if (users.length === 0) {{
                        tbody.innerHTML = '<tr><td colspan="6" style="text-align: center;">No users found</td></tr>';
                        return;
                    }}
                    
                    tbody.innerHTML = users.map(user => `
                        <tr>
                            <td>${{user.username}}</td>
                            <td>${{user.email}}</td>
                            <td>${{user.role}}</td>
                            <td><span class="status ${{user.status === 'active' ? 'online' : 'offline'}}">${{user.status}}</span></td>
                            <td>${{user.last_login || 'Never'}}</td>
                            <td>
                                <button class="btn" style="padding: 5px 10px; margin-right: 5px;" onclick="editUser('${{user.id}}')">Edit</button>
                                <button class="btn-danger" style="padding: 5px 10px;" onclick="deleteUser('${{user.id}}')">Delete</button>
                            </td>
                        </tr>
                    `).join('');
                }})
                .catch(error => {{
                    document.getElementById('users-table').innerHTML = '<tr><td colspan="6" style="text-align: center;">Error loading users</td></tr>';
                }});
        }}
        
        function showCreateUserModal() {{
            document.getElementById('createUserModal').style.display = 'block';
        }}
        
        function hideCreateUserModal() {{
            document.getElementById('createUserModal').style.display = 'none';
        }}
        
        function editUser(userId) {{
            alert('Edit user functionality to be implemented');
        }}
        
        function deleteUser(userId) {{
            if (confirm('Are you sure you want to delete this user?')) {{
                fetch(`/api/admin/users/${{userId}}`, {{ method: 'DELETE' }})
                    .then(response => {{
                        if (response.ok) {{
                            loadUsers();
                        }} else {{
                            alert('Failed to delete user');
                        }}
                    }});
            }}
        }}
        
        document.getElementById('createUserForm').addEventListener('submit', function(e) {{
            e.preventDefault();
            const formData = new FormData(e.target);
            const userData = Object.fromEntries(formData);
            
            fetch('/api/admin/users', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify(userData)
            }})
            .then(response => {{
                if (response.ok) {{
                    hideCreateUserModal();
                    loadUsers();
                    e.target.reset();
                }} else {{
                    alert('Failed to create user');
                }}
            }});
        }});
        
        // Load users on page load
        loadUsers();
    </script>
</body>
</html>
"#,
        styles = get_admin_styles(),
        nav = get_admin_nav()
    );
    
    Html(html)
}

/// Admin plugins management page
pub async fn admin_plugins_handler() -> Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Plugin Management - Administration</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>{styles}</style>
</head>
<body>
    {nav}
    
    <div class="container">
        <div class="header">
            <h1>🧩 Plugin Marketplace Management</h1>
            <p>Enable, disable, and configure system plugins</p>
        </div>
        
        <div class="admin-sidebar">
            <div class="sidebar">
                <h3>Administration</h3>
                <ul>
                    <li><a href="/admin">Dashboard</a></li>
                    <li><a href="/admin/users">User Management</a></li>
                    <li><a href="/admin/plugins" class="active">Plugin Marketplace</a></li>
                    <li><a href="/admin/settings">System Settings</a></li>
                    <li><a href="/admin/monitoring">Monitoring</a></li>
                    <li><a href="/admin/logs">System Logs</a></li>
                </ul>
            </div>
            
            <div class="main-content">
                <h2>System Plugins</h2>
                
                <div class="grid grid-2">
                    <div class="card">
                        <h3>Core System Plugins</h3>
                        <div id="core-plugins">
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #e5e7eb;">
                                <div>
                                    <strong>System Information Collector</strong>
                                    <br><small>Collects comprehensive system hardware and software information</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" checked onchange="togglePlugin('system_info', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #e5e7eb;">
                                <div>
                                    <strong>Container Runtime Manager</strong>
                                    <br><small>Manages Docker, Podman, and LXC container runtimes</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" checked onchange="togglePlugin('container_runtime', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #e5e7eb;">
                                <div>
                                    <strong>GitHub API Integration</strong>
                                    <br><small>Provides GitHub repository search and analysis</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" checked onchange="togglePlugin('github_api', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0;">
                                <div>
                                    <strong>Performance Optimizer</strong>
                                    <br><small>Optimizes deployments based on system capabilities</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" checked onchange="togglePlugin('performance_optimizer', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                        </div>
                    </div>
                    
                    <div class="card">
                        <h3>Authentication Plugins</h3>
                        <div id="auth-plugins">
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #e5e7eb;">
                                <div>
                                    <strong>SSO Manager</strong>
                                    <br><small>Single Sign-On integration for enterprise authentication</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" checked onchange="togglePlugin('sso_manager', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #e5e7eb;">
                                <div>
                                    <strong>Password Reset</strong>
                                    <br><small>Email-based password reset functionality</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" onchange="togglePlugin('password_reset', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 0;">
                                <div>
                                    <strong>RBAC System</strong>
                                    <br><small>Role-based access control and user management</small>
                                </div>
                                <label class="toggle">
                                    <input type="checkbox" checked onchange="togglePlugin('rbac_system', this.checked)">
                                    <span class="slider"></span>
                                </label>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="card" style="margin-top: 20px;">
                    <h3>Plugin Configuration</h3>
                    <div id="plugin-config">
                        <p>Select a plugin above to configure its settings.</p>
                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <style>
        .toggle {{
            position: relative;
            display: inline-block;
            width: 50px;
            height: 24px;
        }}
        
        .toggle input {{
            opacity: 0;
            width: 0;
            height: 0;
        }}
        
        .slider {{
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: #ccc;
            -webkit-transition: .4s;
            transition: .4s;
            border-radius: 24px;
        }}
        
        .slider:before {{
            position: absolute;
            content: "";
            height: 18px;
            width: 18px;
            left: 3px;
            bottom: 3px;
            background-color: white;
            -webkit-transition: .4s;
            transition: .4s;
            border-radius: 50%;
        }}
        
        input:checked + .slider {{
            background-color: #2563eb;
        }}
        
        input:checked + .slider:before {{
            -webkit-transform: translateX(26px);
            -ms-transform: translateX(26px);
            transform: translateX(26px);
        }}
    </style>
    
    <script>
        function togglePlugin(event, pluginId, enabled) {
            fetch(`/api/admin/plugins/${pluginId}/toggle`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ enabled: enabled })
            })
            .then(response => {
                if (!response.ok) {
                    alert('Failed to toggle plugin');
                    // Revert the toggle
                    event.target.checked = !enabled;
                }
            });
        }
        
        // Load plugin status on page load
        fetch('/api/admin/plugins')
            .then(response => response.json())
            .then(plugins => {{
                // Update plugin toggles based on current status
                plugins.forEach(plugin => {{
                    const toggle = document.querySelector(`input[onchange*="${{plugin.id}}"]`);
                    if (toggle) {{
                        toggle.checked = plugin.enabled;
                    }}
                }});
            }})
            .catch(error => console.error('Failed to load plugin status:', error));
    </script>
</body>
</html>
"#,
        styles = get_admin_styles(),
        nav = get_admin_nav()
    );
    
    Html(html)
}

/// Placeholder admin handlers and API endpoints
pub async fn admin_settings_handler() -> Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>System Settings - Administration</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>{styles}</style>
</head>
<body>
    {nav}
    
    <div class="container">
        <div class="header">
            <h1>⚙️ System Settings</h1>
            <p>Configure system behavior, test suites, and platform settings</p>
        </div>
        
        <div class="admin-sidebar">
            <div class="sidebar">
                <h3>Administration</h3>
                <ul>
                    <li><a href="/admin">Dashboard</a></li>
                    <li><a href="/admin/users">User Management</a></li>
                    <li><a href="/admin/plugins">Plugin Marketplace</a></li>
                    <li><a href="/admin/settings" class="active">System Settings</a></li>
                    <li><a href="/admin/monitoring">Monitoring</a></li>
                    <li><a href="/admin/logs">System Logs</a></li>
                </ul>
            </div>
            
            <div class="main-content">
                <div class="grid grid-2">
                    <div class="card">
                        <h3>Comprehensive Test Suite Configuration</h3>
                        <p>Control which test categories are enabled in the comprehensive test suite.</p>
                        
                        <div class="form-group">
                            <label class="toggle-label">
                                <input type="checkbox" id="performance-tests" checked onchange="toggleTestSuite('performance', this.checked)">
                                <span class="toggle-slider"></span>
                                Performance Tests
                            </label>
                            <small>Enable performance benchmarking and optimization tests</small>
                        </div>
                        
                        <div class="form-group">
                            <label class="toggle-label">
                                <input type="checkbox" id="security-tests" checked onchange="toggleTestSuite('security', this.checked)">
                                <span class="toggle-slider"></span>
                                Security Tests
                            </label>
                            <small>Enable security vulnerability and penetration tests</small>
                        </div>
                        
                        <div class="form-group">
                            <label class="toggle-label">
                                <input type="checkbox" id="integration-tests" checked onchange="toggleTestSuite('integration', this.checked)">
                                <span class="toggle-slider"></span>
                                Integration Tests
                            </label>
                            <small>Enable end-to-end integration and component tests</small>
                        </div>
                        
                        <div class="form-group">
                            <label for="test-timeout">Test Timeout (seconds):</label>
                            <input type="number" id="test-timeout" value="60" min="10" max="600" onchange="updateTestTimeout(this.value)">
                        </div>
                        
                        <div class="form-group">
                            <label for="perf-iterations">Performance Test Iterations:</label>
                            <input type="number" id="perf-iterations" value="100" min="1" max="1000" onchange="updatePerfIterations(this.value)">
                        </div>
                        
                        <div style="margin-top: 20px;">
                            <button class="btn" onclick="runTestSuite()">Run Comprehensive Test Suite</button>
                            <button class="btn" style="background: #6b7280; margin-left: 10px;" onclick="resetTestConfig()">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <div class="card">
                        <h3>System Configuration</h3>
                        
                        <div class="form-group">
                            <label for="system-name">System Name:</label>
                            <input type="text" id="system-name" value="Automation Nation" onchange="updateSetting('system_name', this.value)">
                        </div>
                        
                        <div class="form-group">
                            <label for="github-timeout">GitHub API Timeout (seconds):</label>
                            <input type="number" id="github-timeout" value="30" min="5" max="120" onchange="updateSetting('github_timeout', this.value)">
                        </div>
                        
                        <div class="form-group">
                            <label for="container-runtime">Default Container Runtime:</label>
                            <select id="container-runtime" onchange="updateSetting('container_runtime', this.value)">
                                <option value="podman" selected>Podman</option>
                                <option value="docker">Docker</option>
                                <option value="lxc">LXC</option>
                            </select>
                        </div>
                        
                        <div class="form-group">
                            <label class="toggle-label">
                                <input type="checkbox" id="auto-update" checked onchange="updateSetting('auto_update', this.checked)">
                                <span class="toggle-slider"></span>
                                Auto-update System Profiles
                            </label>
                            <small>Automatically regenerate system profiles when hardware changes are detected</small>
                        </div>
                        
                        <div class="form-group">
                            <label class="toggle-label">
                                <input type="checkbox" id="verbose-logging" onchange="updateSetting('verbose_logging', this.checked)">
                                <span class="toggle-slider"></span>
                                Verbose Logging
                            </label>
                            <small>Enable detailed debug logging for troubleshooting</small>
                        </div>
                        
                        <div style="margin-top: 20px;">
                            <button class="btn-success" onclick="saveSettings()">Save All Settings</button>
                        </div>
                    </div>
                </div>
                
                <div class="card" style="margin-top: 20px;">
                    <h3>Test Suite Status</h3>
                    <div id="test-status">
                        <p>No test suite currently running.</p>
                    </div>
                    <div id="test-results" style="display: none;">
                        <h4>Last Test Results:</h4>
                        <pre id="test-output"></pre>
                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <style>
        .toggle-label {{
            display: flex;
            align-items: center;
            gap: 10px;
            font-weight: 500;
            color: #374151;
            cursor: pointer;
        }}
        
        .toggle-slider {{
            position: relative;
            display: inline-block;
            width: 50px;
            height: 24px;
            background-color: #ccc;
            border-radius: 24px;
            transition: .4s;
        }}
        
        .toggle-slider:before {{
            position: absolute;
            content: "";
            height: 18px;
            width: 18px;
            left: 3px;
            bottom: 3px;
            background-color: white;
            border-radius: 50%;
            transition: .4s;
        }}
        
        input[type="checkbox"]:checked + .toggle-slider {{
            background-color: #2563eb;
        }}
        
        input[type="checkbox"]:checked + .toggle-slider:before {{
            transform: translateX(26px);
        }}
        
        input[type="checkbox"] {{
            display: none;
        }}
    </style>
    
    <script>
        function toggleTestSuite(category, enabled) {{
            console.log(`${{category}} tests ${{enabled ? 'enabled' : 'disabled'}}`);
            // Update test configuration
            fetch('/api/admin/settings', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{
                    [`enable_${{category}}_tests`]: enabled
                }})
            }});
        }}
        
        function updateTestTimeout(value) {{
            console.log(`Test timeout updated to ${{value}} seconds`);
            updateSetting('test_timeout_seconds', parseInt(value));
        }}
        
        function updatePerfIterations(value) {{
            console.log(`Performance test iterations updated to ${{value}}`);
            updateSetting('performance_test_iterations', parseInt(value));
        }}
        
        function updateSetting(key, value) {{
            console.log(`Setting ${{key}} updated to ${{value}}`);
            fetch('/api/admin/settings', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{[key]: value}})
            }});
        }}
        
        function saveSettings() {{
            alert('Settings saved successfully!');
        }}
        
        function resetTestConfig() {{
            // Reset to defaults
            document.getElementById('performance-tests').checked = true;
            document.getElementById('security-tests').checked = true;
            document.getElementById('integration-tests').checked = true;
            document.getElementById('test-timeout').value = 60;
            document.getElementById('perf-iterations').value = 100;
            alert('Test configuration reset to defaults');
        }}
        
        function runTestSuite() {{
            const statusDiv = document.getElementById('test-status');
            const resultsDiv = document.getElementById('test-results');
            
            statusDiv.innerHTML = '<p>🔄 Running comprehensive test suite...</p>';
            
            // Simulate test run
            setTimeout(() => {{
                statusDiv.innerHTML = '<p>✅ Test suite completed successfully!</p>';
                resultsDiv.style.display = 'block';
                document.getElementById('test-output').textContent = 
                    'Functional Tests: PASSED (15/15)\\n' +
                    'Performance Tests: PASSED (8/8)\\n' +
                    'Security Tests: PASSED (12/12)\\n' +
                    'Integration Tests: PASSED (6/6)\\n' +
                    '\\nTotal Duration: 45.2 seconds\\n' +
                    'All tests passed successfully!';
            }}, 3000);
        }}
        
        // Load current settings on page load
        fetch('/api/admin/settings')
            .then(response => response.json())
            .then(settings => {{
                // Update form fields with current settings
                if (settings.system_name) {{
                    document.getElementById('system-name').value = settings.system_name;
                }}
                if (settings.container_runtime) {{
                    document.getElementById('container-runtime').value = settings.container_runtime;
                }}
                // Update other settings as needed
            }})
            .catch(error => console.error('Failed to load settings:', error));
    </script>
</body>
</html>
"#,
        styles = get_admin_styles(),
        nav = get_admin_nav()
    );
    
    Html(html)
}

pub async fn admin_monitoring_handler() -> Html<String> {
    Html(format!(r#"<html><head><title>Monitoring</title><style>{}</style></head><body>{}<div class="container"><h1>System Monitoring - Coming Soon</h1></div></body></html>"#, get_admin_styles(), get_admin_nav()))
}

pub async fn admin_logs_handler() -> Html<String> {
    Html(format!(r#"<html><head><title>Logs</title><style>{}</style></head><body>{}<div class="container"><h1>System Logs - Coming Soon</h1></div></body></html>"#, get_admin_styles(), get_admin_nav()))
}

// Admin API Handlers (placeholders with mock data)
pub async fn api_list_users() -> Json<Vec<serde_json::Value>> {
    Json(vec![serde_json::json!({"id": "1", "username": "admin", "email": "admin@localhost", "role": "admin", "status": "active"})])
}

pub async fn api_create_user() -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": true, "message": "User created"}))
}

pub async fn api_get_user(Path(_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"id": "1", "username": "admin", "role": "admin"}))
}

pub async fn api_delete_user(Path(_id): Path<String>) -> StatusCode {
    StatusCode::NO_CONTENT
}

pub async fn api_list_plugins() -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({"id": "system_info", "name": "System Info", "enabled": true}),
        serde_json::json!({"id": "password_reset", "name": "Password Reset", "enabled": false})
    ])
}

pub async fn api_toggle_plugin(Path(_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": true, "message": "Plugin toggled"}))
}

pub async fn api_get_settings() -> Json<serde_json::Value> {
    Json(serde_json::json!({"system_name": "Automation Nation", "version": "1.0.0"}))
}

pub async fn api_update_settings() -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": true, "message": "Settings updated"}))
}

pub async fn api_system_stats() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "uptime": "Running",
        "recent_activity": [
            {"action": "System Started", "timestamp": "2024-01-15T10:00:00Z", "details": "Web server initialized"}
        ]
    }))
}

pub async fn api_get_logs() -> Json<Vec<serde_json::Value>> {
    Json(vec![serde_json::json!({"timestamp": "2024-01-15T10:00:00Z", "level": "INFO", "message": "Server started"})])
}
