use axum::{
    extract::{Multipart, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use base64::{Engine as _, engine::general_purpose};
use std::sync::Arc;
use oauth2::{
    basic::BasicClient,
    AuthorizationCode, CsrfToken, Scope, TokenResponse,
    reqwest::async_http_client,
};

use crate::github::GitHubClient;
use crate::templates::{IndexTemplate, DashboardTemplate};

#[derive(Deserialize)]
pub struct LoginParams {
    token: String,
}

pub async fn index(jar: CookieJar) -> impl IntoResponse {
    if jar.get("gh_token").is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    let template = IndexTemplate { 
        error: None,
        version: crate::ASSET_VERSION.to_string(),
    };
    Html(template.to_string()).into_response()
}

pub async fn login(
    jar: CookieJar,
    Form(params): Form<LoginParams>,
) -> impl IntoResponse {
    let client = GitHubClient::new(params.token.clone());
    match client.validate_token().await {
        Ok(_) => {
            let mut cookie = Cookie::new("gh_token", params.token);
            cookie.set_path("/");
            cookie.set_http_only(true);
            // In a real app, set secure, same_site, etc.
            
            (jar.add(cookie), Redirect::to("/dashboard")).into_response()
        }
        Err(_) => {
            let template = IndexTemplate {
                error: Some("Invalid GitHub Token".to_string()),
                version: crate::ASSET_VERSION.to_string(),
            };
            Html(template.to_string()).into_response()
        }
    }
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove(Cookie::build("gh_token")), Redirect::to("/")).into_response()
}

// OAuth callback query parameters
#[derive(Deserialize)]
pub struct AuthCallbackParams {
    code: String,
    #[allow(dead_code)]
    state: String, // CSRF token - kept for future validation
}

// GitHub OAuth login - redirect to GitHub
pub async fn auth_github(
    State(oauth_client): State<Arc<Option<BasicClient>>>,
) -> impl IntoResponse {
    let client = match oauth_client.as_ref() {
        Some(c) => c,
        None => {
            return Redirect::to("/?error=oauth_disabled").into_response();
        }
    };

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("repo".to_string()))
        .url();

    Redirect::to(auth_url.as_str()).into_response()
}

// OAuth callback - exchange code for token
pub async fn auth_callback(
    Query(params): Query<AuthCallbackParams>,
    jar: CookieJar,
    State(oauth_client): State<Arc<Option<BasicClient>>>,
) -> impl IntoResponse {
    let client = match oauth_client.as_ref() {
        Some(c) => c,
        None => {
            return Redirect::to("/?error=oauth_disabled").into_response();
        }
    };

    // Exchange code for access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    match token_result {
        Ok(token) => {
            let access_token = token.access_token().secret().to_string();
            
            // Validate token by fetching user info
            let gh_client = GitHubClient::new(access_token.clone());
            match gh_client.validate_token().await {
                Ok(_) => {
                    let mut cookie = Cookie::new("gh_token", access_token);
                    cookie.set_path("/");
                    cookie.set_http_only(true);
                    (jar.add(cookie), Redirect::to("/dashboard")).into_response()
                }
                Err(_) => {
                    Redirect::to("/?error=token_validation_failed").into_response()
                }
            }
        }
        Err(e) => {
            println!("OAuth token exchange error: {:?}", e);
            Redirect::to("/?error=oauth_failed").into_response()
        }
    }
}

pub async fn dashboard(jar: CookieJar) -> impl IntoResponse {
    let token = match jar.get("gh_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return Redirect::to("/").into_response(),
    };

    let client = GitHubClient::new(token);
    let username = match client.validate_token().await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/logout").into_response(),
    };

    // Auto-create GitHub Pages repository if it doesn't exist
    let pages_repo = format!("{}.github.io", username);
    let repo_exists = client.check_repository_exists(&username, &pages_repo).await.unwrap_or(false);

    if !repo_exists {
        // Try to create the GitHub Pages repository
        let _ = client.create_repository(&pages_repo, "GitHub Pages - Image Storage").await;
    }

    // Read upload result from cookie (if exists)
    let mut uploaded_link = None;
    let mut pages_link = None;
    let mut error = None;
    let mut new_jar = jar.clone();

    if let Some(result_cookie) = jar.get("upload_result") {
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(result_cookie.value()) {
            uploaded_link = result.get("cdn_link").and_then(|v| v.as_str()).map(|s| s.to_string());
            pages_link = result.get("pages_link").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        // Remove the cookie after reading
        new_jar = new_jar.remove(Cookie::build("upload_result"));
    }

    if let Some(error_cookie) = jar.get("upload_error") {
        error = Some(error_cookie.value().to_string());
        // Remove the cookie after reading
        new_jar = new_jar.remove(Cookie::build("upload_error"));
    }

    // List images from the repository
    let images = client.list_images(&username, &pages_repo, "").await.unwrap_or_default();

    // Always provide pages_link since we enforce the Pages repo
    if pages_link.is_none() {
        pages_link = Some(format!("https://{}", pages_repo));
    }

    let template = DashboardTemplate {
        username,
        repo: Some(pages_repo),
        uploaded_link,
        pages_link,
        images,
        error,
        version: crate::ASSET_VERSION.to_string(),
    };

    (new_jar, Html(template.to_string())).into_response()
}

pub async fn upload(
    jar: CookieJar,
    mut multipart: Multipart,
) -> Response {
    // Early validation - create error cookie and redirect if needed
    let token = match jar.get("gh_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return (jar, Redirect::to("/")).into_response();
        }
    };

    let client = GitHubClient::new(token.clone());
    let username = match client.validate_token().await {
        Ok(u) => u,
        Err(_) => {
            return (jar, Redirect::to("/logout")).into_response();
        }
    };

    let mut repo = None;
    let mut path_prefix = String::new();
    let mut file_content = Vec::new();
    let mut filename = String::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "repo" {
            repo = Some(field.text().await.unwrap_or_default());
        } else if name == "path" {
            path_prefix = field.text().await.unwrap_or_default();
        } else if name == "file" {
            filename = field.file_name().unwrap_or("").to_string();
            
            // Use streaming/chunked reading for better compatibility
            let mut buffer = Vec::new();
            let mut temp_field = field;
            
            while let Some(chunk) = temp_field.chunk().await.transpose() {
                match chunk {
                    Ok(data) => {
                        buffer.extend_from_slice(&data);
                    }
                    Err(e) => {
                        println!("Error reading chunk: {}", e);
                        let mut cookie = Cookie::new("upload_error", format!("Error reading file chunk: {}", e));
                        cookie.set_path("/");
                        cookie.set_http_only(true);
                        return (jar.add(cookie), Redirect::to("/dashboard")).into_response();
                    }
                }
            }
            
            file_content = buffer;
            println!("Read file '{}': {} bytes (via chunks)", filename, file_content.len());
        }
    }

    // Validate file content
    if file_content.is_empty() {
        let mut cookie = Cookie::new("upload_error", "Failed to read file content. The file may be empty or corrupted.");
        cookie.set_path("/");
        cookie.set_http_only(true);
        return (jar.add(cookie), Redirect::to("/dashboard")).into_response();
    }

    if filename.is_empty() {
        let mut cookie = Cookie::new("upload_error", "No file selected.");
        cookie.set_path("/");
        cookie.set_http_only(true);
        return (jar.add(cookie), Redirect::to("/dashboard")).into_response();
    }

    // Log file info for debugging
    let file_size_mb = file_content.len() as f64 / 1024.0 / 1024.0;
    println!("Uploading file: {} ({:.2} MB)", filename, file_size_mb);

    // Warn if file is large
    if file_size_mb > 50.0 {
        let mut cookie = Cookie::new("upload_error", "File too large. Maximum size is 50MB.");
        cookie.set_path("/");
        cookie.set_http_only(true);
        return (jar.add(cookie), Redirect::to("/dashboard")).into_response();
    }

    let repo = match repo {
        Some(r) => r,
        None => {
            let pages_repo = format!("{}.github.io", username);
            // Default to pages repo if not provided
            pages_repo
        }
    };

    // Auto-prepend username if repo doesn't contain "/"
    let full_repo = if repo.contains('/') {
        repo.clone()
    } else {
        format!("{}/{}", username, repo)
    };

    // Clean up path and add timestamp to avoid collisions
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis(); // Use milliseconds for more precision

    // Add timestamp before file extension
    let filename_with_timestamp = if let Some(pos) = filename.rfind('.') {
        format!("{}_{}{}", &filename[..pos], timestamp, &filename[pos..])
    } else {
        format!("{}_{}", filename, timestamp)
    };

    let full_path = if path_prefix.is_empty() {
        filename_with_timestamp
    } else {
        format!("{}/{}", path_prefix.trim_end_matches('/'), filename_with_timestamp)
    };

    let content_base64 = general_purpose::STANDARD.encode(&file_content);

    // Parse owner/repo
    let parts: Vec<&str> = full_repo.split('/').collect();
    if parts.len() != 2 {
        let mut cookie = Cookie::new("upload_error", "Invalid repository format. Use user/repo");
        cookie.set_path("/");
        cookie.set_http_only(true);
        return (jar.add(cookie), Redirect::to("/dashboard")).into_response();
    }
    let owner = parts[0];
    let repo_name = parts[1];

    // Check if repository exists, create if not
    let repo_exists = client.check_repository_exists(owner, repo_name).await.unwrap_or(false);
    if !repo_exists {
        // Try to create the repository
        let _ = client.create_repository(repo_name, "Image Storage via RustPic").await;
    }

    match client.upload_file_with_links(owner, repo_name, &full_path, content_base64, file_content.len()).await {
        Ok(result) => {
            // Store upload result in cookie temporarily
            let result_json = serde_json::json!({
                "cdn_link": result.cdn_link,
                "pages_link": result.pages_link
            }).to_string();

            let mut cookie = Cookie::new("upload_result", result_json);
            cookie.set_path("/");
            cookie.set_http_only(true);

            // Redirect to dashboard to prevent form resubmission
            (jar.add(cookie), Redirect::to("/dashboard")).into_response()
        }
        Err(e) => {
            // Store error in cookie
            let mut cookie = Cookie::new("upload_error", format!("Upload failed: {}", e));
            cookie.set_path("/");
            cookie.set_http_only(true);

            // Redirect to dashboard
            (jar.add(cookie), Redirect::to("/dashboard")).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteParams {
    repo: String,
    path: String,
    sha: String,
}

pub async fn delete_image(
    jar: CookieJar,
    Form(params): Form<DeleteParams>,
) -> impl IntoResponse {
    let token = match jar.get("gh_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return Redirect::to("/").into_response(),
    };

    let client = GitHubClient::new(token);
    let username = match client.validate_token().await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/logout").into_response(),
    };

    // Verify ownership (simple check: repo starts with username)
    // Or just let GitHub API handle permission errors
    let parts: Vec<&str> = params.repo.split('/').collect();
    let owner = if parts.len() == 2 { parts[0] } else { &username };
    let repo_name = if parts.len() == 2 { parts[1] } else { &params.repo };

    match client.delete_file(owner, repo_name, &params.path, &params.sha).await {
        Ok(_) => {
            // Redirect back to dashboard
            Redirect::to("/dashboard").into_response()
        }
        Err(e) => {
            let mut cookie = Cookie::new("upload_error", format!("Delete failed: {}", e));
            cookie.set_path("/");
            cookie.set_http_only(true);
            (jar.add(cookie), Redirect::to("/dashboard")).into_response()
        }
    }
}
