//! GitHub API client for fetching open source software information

use crate::web_types::*;
use crate::Result;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use log::{debug, info, warn};

/// GitHub API client
pub struct GitHubApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl GitHubApiClient {
    /// Create a new GitHub API client
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.github.com".to_string(),
            token,
        }
    }

    /// Search for repositories based on query
    pub async fn search_repositories(&self, request: &SearchRepositoriesRequest) -> Result<SearchRepositoriesResponse> {
        let mut query = request.query.clone();
        
        // Add language filter if specified
        if let Some(language) = &request.language {
            query.push_str(&format!(" language:{}", language));
        }
        
        // Build query parameters
        let per_page_str = request.per_page.unwrap_or(30).to_string();
        let page_str = request.page.unwrap_or(1).to_string();
        
        let params = vec![
            ("q", query.as_str()),
            ("sort", request.sort.as_deref().unwrap_or("stars")),
            ("order", request.order.as_deref().unwrap_or("desc")),
            ("per_page", &per_page_str),
            ("page", &page_str),
        ];

        let url = format!("{}/search/repositories", self.base_url);
        
        let mut req_builder = self.client.get(&url).query(&params);
        
        // Add authentication if token is available
        if let Some(token) = &self.token {
            req_builder = req_builder.header("Authorization", format!("token {}", token));
        }
        
        req_builder = req_builder.header("User-Agent", "Automation_nation/1.0");
        
        debug!("Searching repositories with query: {}", query);
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("GitHub API error {}: {}", status, error_text));
        }
        
        let data: Value = response.json().await?;
        
        let repositories: Vec<GitHubRepository> = data["items"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|item| self.parse_repository(item).ok())
            .collect();
            
        let total_count = data["total_count"].as_u64().unwrap_or(0) as u32;
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(30);
        let has_more = (page * per_page) < total_count;
        
        info!("Found {} repositories (page {}/{})", repositories.len(), page, (total_count + per_page - 1) / per_page);
        
        Ok(SearchRepositoriesResponse {
            repositories,
            total_count,
            page,
            per_page,
            has_more,
        })
    }

    /// Get detailed information about a specific repository
    pub async fn get_repository(&self, owner: &str, name: &str) -> Result<GitHubRepository> {
        let url = format!("{}/repos/{}/{}", self.base_url, owner, name);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.header("Authorization", format!("token {}", token));
        }
        
        req_builder = req_builder.header("User-Agent", "Automation_nation/1.0");
        
        debug!("Fetching repository details: {}/{}", owner, name);
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("GitHub API error {}: {}", status, error_text));
        }
        
        let data: Value = response.json().await?;
        self.parse_repository(&data)
    }

    /// Get repository languages
    pub async fn get_repository_languages(&self, owner: &str, name: &str) -> Result<HashMap<String, u64>> {
        let url = format!("{}/repos/{}/{}/languages", self.base_url, owner, name);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.header("Authorization", format!("token {}", token));
        }
        
        req_builder = req_builder.header("User-Agent", "Automation_nation/1.0");
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            warn!("Failed to fetch languages for {}/{}: {}", owner, name, response.status());
            return Ok(HashMap::new());
        }
        
        let languages: HashMap<String, u64> = response.json().await.unwrap_or_default();
        Ok(languages)
    }

    /// Search for repositories by topic
    pub async fn search_by_topic(&self, topic: &str, limit: u32) -> Result<Vec<GitHubRepository>> {
        let request = SearchRepositoriesRequest {
            query: format!("topic:{}", topic),
            language: None,
            sort: Some("stars".to_string()),
            order: Some("desc".to_string()),
            per_page: Some(limit),
            page: Some(1),
        };
        
        let response = self.search_repositories(&request).await?;
        Ok(response.repositories)
    }

    /// Get trending repositories by language
    pub async fn get_trending_repositories(&self, language: Option<String>, limit: u32) -> Result<Vec<GitHubRepository>> {
        let mut query = "created:>2023-01-01".to_string();
        
        if let Some(lang) = language {
            query.push_str(&format!(" language:{}", lang));
        }
        
        let request = SearchRepositoriesRequest {
            query,
            language: None,
            sort: Some("stars".to_string()),
            order: Some("desc".to_string()),
            per_page: Some(limit),
            page: Some(1),
        };
        
        let response = self.search_repositories(&request).await?;
        Ok(response.repositories)
    }

    /// Parse a repository from GitHub API response
    fn parse_repository(&self, data: &Value) -> Result<GitHubRepository> {
        let id = data["id"].as_u64().ok_or_else(|| anyhow::anyhow!("Missing repository id"))?;
        let name = data["name"].as_str().ok_or_else(|| anyhow::anyhow!("Missing repository name"))?.to_string();
        let full_name = data["full_name"].as_str().ok_or_else(|| anyhow::anyhow!("Missing repository full_name"))?.to_string();
        
        let description = data["description"].as_str().map(|s| s.to_string());
        let html_url = data["html_url"].as_str().unwrap_or_default().to_string();
        let clone_url = data["clone_url"].as_str().unwrap_or_default().to_string();
        let ssh_url = data["ssh_url"].as_str().unwrap_or_default().to_string();
        let language = data["language"].as_str().map(|s| s.to_string());
        let languages_url = data["languages_url"].as_str().unwrap_or_default().to_string();
        
        let stargazers_count = data["stargazers_count"].as_u64().unwrap_or(0) as u32;
        let forks_count = data["forks_count"].as_u64().unwrap_or(0) as u32;
        let open_issues_count = data["open_issues_count"].as_u64().unwrap_or(0) as u32;
        
        let topics = data["topics"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();
            
        let license = data["license"].as_object().and_then(|l| {
            Some(GitHubLicense {
                key: l["key"].as_str()?.to_string(),
                name: l["name"].as_str()?.to_string(),
                spdx_id: l["spdx_id"].as_str().map(|s| s.to_string()),
            })
        });
        
        let created_at = chrono::DateTime::parse_from_rfc3339(
            data["created_at"].as_str().unwrap_or("2020-01-01T00:00:00Z")
        ).unwrap_or_default().with_timezone(&chrono::Utc);
        
        let updated_at = chrono::DateTime::parse_from_rfc3339(
            data["updated_at"].as_str().unwrap_or("2020-01-01T00:00:00Z")
        ).unwrap_or_default().with_timezone(&chrono::Utc);
        
        let default_branch = data["default_branch"].as_str().unwrap_or("main").to_string();
        
        Ok(GitHubRepository {
            id,
            name,
            full_name,
            description,
            html_url,
            clone_url,
            ssh_url,
            language,
            languages_url,
            stargazers_count,
            forks_count,
            open_issues_count,
            topics,
            license,
            created_at,
            updated_at,
            default_branch,
            exposed_port: None,  // Set to None as default, can be configured later
            health_check_path: None,  // Set to None as default, can be configured later
        })
    }

    /// Get popular repositories by category
    pub async fn get_popular_by_category(&self, category: &str, limit: u32) -> Result<Vec<GitHubRepository>> {
        let query = match category.to_lowercase().as_str() {
            "web" => "topic:web topic:framework OR topic:website",
            "api" => "topic:api topic:rest OR topic:graphql",
            "database" => "topic:database topic:sql OR topic:nosql",
            "monitoring" => "topic:monitoring topic:metrics OR topic:observability",
            "security" => "topic:security topic:authentication OR topic:encryption",
            "devops" => "topic:devops topic:deployment OR topic:infrastructure",
            "ml" => "topic:machine-learning topic:ai OR topic:deep-learning",
            "blockchain" => "topic:blockchain topic:cryptocurrency OR topic:ethereum",
            "game" => "topic:game topic:gaming OR topic:gamedev",
            _ => category,
        };
        
        let request = SearchRepositoriesRequest {
            query: query.to_string(),
            language: None,
            sort: Some("stars".to_string()),
            order: Some("desc".to_string()),
            per_page: Some(limit),
            page: Some(1),
        };
        
        let response = self.search_repositories(&request).await?;
        Ok(response.repositories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[tokio::test]
    async fn test_github_client_creation() {
        let client = GitHubApiClient::new(None);
        assert_eq!(client.base_url, "https://api.github.com");
        assert!(client.token.is_none());
    }

    #[tokio::test]
    async fn test_github_client_with_token() {
        let token = "test_token".to_string();
        let client = GitHubApiClient::new(Some(token.clone()));
        assert_eq!(client.token, Some(token));
    }

    #[tokio::test]
    async fn test_search_request_building() {
        let request = SearchRepositoriesRequest {
            query: "rust".to_string(),
            language: Some("rust".to_string()),
            sort: Some("stars".to_string()),
            order: Some("desc".to_string()),
            per_page: Some(10),
            page: Some(1),
        };
        
        assert_eq!(request.query, "rust");
        assert_eq!(request.language, Some("rust".to_string()));
    }
}