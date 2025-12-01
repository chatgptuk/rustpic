use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub error: Option<String>,
    pub version: String,
}

use crate::github::FileInfo;

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub username: String,
    pub repo: Option<String>,
    pub uploaded_link: Option<String>,
    pub pages_link: Option<String>,
    pub images: Vec<FileInfo>,
    pub error: Option<String>,
    pub version: String,
}
