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
        .nav-links {{ max-width: 1200px; margin: 0 auto; padding: 0 20px; }}
        .nav-links a {{ color: white; text-decoration: none; margin-right: 20px; }}
        .nav-links a:hover {{ color: #60a5fa; }}
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