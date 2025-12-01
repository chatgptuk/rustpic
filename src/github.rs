use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone)]
pub struct GitHubClient {
    client: Client,
    token: String,
}

#[derive(Serialize)]
struct UploadRequest {
    message: String,
    content: String, // Base64 encoded
}

#[derive(Serialize)]
struct CreateRepoRequest {
    name: String,
    description: String,
    auto_init: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub download_url: Option<String>,
}

pub struct UploadResult {
    pub cdn_link: String,
    pub pages_link: Option<String>,
}

#[derive(Deserialize)]
struct User {
    login: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        let client = Client::builder()
            .user_agent("rustpic")
            .build()
            .unwrap();
        Self { client, token }
    }

    pub async fn validate_token(&self) -> Result<String, Box<dyn Error>> {
        let url = "https://api.github.com/user";
        let resp = self.client
            .get(url)
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;

        if resp.status().is_success() {
            let user: User = resp.json().await?;
            Ok(user.login)
        } else {
            Err("Invalid token".into())
        }
    }

    #[allow(dead_code)]
    pub async fn upload_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content_base64: String,
    ) -> Result<String, Box<dyn Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
        
        let body = UploadRequest {
            message: format!("Upload {} via RustPic", path),
            content: content_base64,
        };

        let resp = self.client
            .put(&url)
            .header("Authorization", format!("token {}", self.token))
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            // Don't try to parse the response body - we don't need it
            // Just construct the CDN link from the path
            let cdn_link = format!("https://cdn.jsdelivr.net/gh/{}/{}/{}", owner, repo, path);
            Ok(cdn_link)
        } else {
            let error_text = resp.text().await?;
            Err(format!("Upload failed: {}", error_text).into())
        }
    }

    pub async fn check_repository_exists(&self, owner: &str, repo: &str) -> Result<bool, Box<dyn Error>> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;
        
        Ok(resp.status().is_success())
    }

    pub async fn create_repository(&self, name: &str, description: &str) -> Result<(), Box<dyn Error>> {
        let url = "https://api.github.com/user/repos";
        let body = CreateRepoRequest {
            name: name.to_string(),
            description: description.to_string(),
            auto_init: true,
        };

        let resp = self.client
            .post(url)
            .header("Authorization", format!("token {}", self.token))
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let error_text = resp.text().await?;
            Err(format!("Repo creation failed: {}", error_text).into())
        }
    }

    pub async fn list_images(&self, owner: &str, repo: &str, path: &str) -> Result<Vec<FileInfo>, Box<dyn Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;

        if resp.status().is_success() {
            let mut files: Vec<FileInfo> = resp.json().await?;
            
            // No filtering - show all files
            
            // Sort by timestamp extracted from filename (newest first)
            // Format: filename_timestamp.ext
            files.sort_by(|a, b| {
                fn extract_timestamp(name: &str) -> u128 {
                    // Find extension
                    if let Some(dot_pos) = name.rfind('.') {
                        let name_no_ext = &name[..dot_pos];
                        // Find last underscore
                        if let Some(underscore_pos) = name_no_ext.rfind('_') {
                            if let Ok(ts) = name_no_ext[underscore_pos + 1..].parse::<u128>() {
                                return ts;
                            }
                        }
                    }
                    0 // Fallback for files without timestamp
                }

                let ts_a = extract_timestamp(&a.name);
                let ts_b = extract_timestamp(&b.name);

                if ts_a != 0 && ts_b != 0 {
                    // Sort descending (newest first)
                    ts_b.cmp(&ts_a)
                } else {
                    // Fallback to name sort
                    a.name.cmp(&b.name)
                }
            });
            
            // Take first 100
            Ok(files.into_iter().take(100).collect())
        } else {
            Ok(vec![])
        }
    }

    pub async fn delete_file(&self, owner: &str, repo: &str, path: &str, sha: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
        
        #[derive(Serialize)]
        struct DeleteRequest {
            message: String,
            sha: String,
        }

        let body = DeleteRequest {
            message: format!("Delete {} via RustPic", path),
            sha: sha.to_string(),
        };

        let resp = self.client
            .delete(&url)
            .header("Authorization", format!("token {}", self.token))
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let error_text = resp.text().await?;
            Err(format!("Delete failed: {}", error_text).into())
        }
    }

    pub async fn upload_file_with_links(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content_base64: String,
        file_size_bytes: usize,
    ) -> Result<UploadResult, Box<dyn Error>> {
        // First upload the file
        let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
        
        let body = UploadRequest {
            message: format!("Upload {} via RustPic", path),
            content: content_base64,
        };

        let resp = self.client
            .put(&url)
            .header("Authorization", format!("token {}", self.token))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(format!("Upload failed: {}", error_text).into());
        }

        // Construct CDN link only if file is under 20MB (jsDelivr limit)
        let cdn_link = if file_size_bytes <= 20 * 1024 * 1024 {
            format!("https://cdn.jsdelivr.net/gh/{}/{}/{}", owner, repo, path)
        } else {
            // For files >20MB, use GitHub raw URL
            format!("https://raw.githubusercontent.com/{}/{}/main/{}", owner, repo, path)
        };
        
        // Check if GitHub Pages repo exists (username.github.io)
        let pages_repo = format!("{}.github.io", owner);
        let pages_link = if self.check_repository_exists(owner, &pages_repo).await.unwrap_or(false) && repo == pages_repo {
            Some(format!("https://{}/{}", pages_repo, path))
        } else {
            None
        };

        Ok(UploadResult { cdn_link, pages_link })
    }
}
