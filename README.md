# RustPic - GitHub Image Hosting / GitHub 图床

RustPic is a lightweight, high-performance image hosting solution written in Rust. It leverages your GitHub repository as storage, providing a fast and free way to host images and files for your blogs, websites, or personal use.

RustPic 是一个基于 Rust 编写的轻量级、高性能图床解决方案。它利用你的 GitHub 仓库作为存储后端，为你的博客、网站或个人使用提供快速且免费的图片和文件托管服务。

![RustPic Dashboard](assets/favicon.svg)

## Features / 功能特性

-   🚀 **High Performance**: Built with Rust and Axum for blazing fast speeds.
    -   🚀 **高性能**：基于 Rust 和 Axum 构建，速度极快。
-   📦 **GitHub Storage**: Uses your GitHub repository for unlimited, free storage.
    -   📦 **GitHub 存储**：使用 GitHub 仓库作为存储，无限且免费。
-   ⚡ **CDN Acceleration**: Automatically generates jsDelivr CDN links for fast global access.
    -   ⚡ **CDN 加速**：自动生成 jsDelivr CDN 链接，实现全球快速访问。
-   📂 **File Support**: Supports uploading images (JPG, PNG, GIF, WEBP, HEIC) and other files (PDF, ZIP, etc.).
    -   📂 **文件支持**：支持上传图片（JPG, PNG, GIF, WEBP, HEIC）和其他文件（PDF, ZIP 等）。
-   🔒 **Secure**: Supports both GitHub OAuth and Personal Access Token (PAT) authentication.
    -   🔒 **安全可靠**：支持 GitHub OAuth 和个人访问令牌 (PAT) 两种认证方式。
-   📱 **Responsive UI**: Beautiful, glassmorphism-inspired UI that works perfectly on desktop and mobile.
    -   📱 **响应式界面**：精美的玻璃拟态 UI 设计，完美适配桌面和移动端。
-   🛠️ **Auto Configuration**: Automatically creates storage repositories and GitHub Pages branches if needed.
    -   🛠️ **自动配置**：需要时自动创建存储仓库和 GitHub Pages 分支。

## Installation / 安装

### Prerequisites / 前置要求

-   Rust (latest stable version)
-   A GitHub account

### Build from Source / 源码编译

1.  Clone the repository:
    ```bash
    git clone https://github.com/yourusername/rustpic.git
    cd rustpic
    ```

2.  Build and run:
    ```bash
    cargo run --release
    ```

3.  Open your browser at `http://localhost:3002`.

## Configuration / 配置

You can configure RustPic using environment variables or a `.env` file.
你可以通过环境变量或 `.env` 文件配置 RustPic。

| Variable | Description | Required |
| :--- | :--- | :--- |
| `GITHUB_CLIENT_ID` | GitHub OAuth Client ID | No (if using PAT) |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth Client Secret | No (if using PAT) |
| `OAUTH_CALLBACK_URL` | OAuth Callback URL (default: `http://localhost:3002/auth/callback`) | No |

## Usage / 使用方法

1.  **Login**: Use "Continue with GitHub" (if configured) or enter your GitHub Personal Access Token (PAT).
    -   **登录**：使用 "Continue with GitHub"（如果已配置）或输入你的 GitHub 个人访问令牌 (PAT)。
2.  **Upload**: Drag and drop files or click to select. Supports files up to 50MB.
    -   **上传**：拖拽文件或点击选择。支持最大 50MB 的文件。
3.  **Manage**: View your uploaded files, copy CDN links, or delete files directly from the dashboard.
    -   **管理**：查看已上传的文件，复制 CDN 链接，或直接在仪表盘中删除文件。

## License / 许可证

MIT License
