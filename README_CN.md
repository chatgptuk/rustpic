# RustPic - GitHub 图床

[**English**](README.md) | [**中文说明**](README_CN.md)

RustPic 是一个基于 Rust 编写的轻量级、高性能图床解决方案。它利用你的 GitHub 仓库作为存储后端，为你的博客、网站或个人使用提供快速且免费的图片和文件托管服务。

![RustPic Dashboard](assets/favicon.svg)

## 界面预览

### 登录页
![登录页](https://lyduwss.github.io/2_1764628787646.png/1_1764628754790.png)

### 仪表盘
![仪表盘](https://lyduwss.github.io/2_1764628787646.png/2_1764628787646.png)

## 功能特性

-   🚀 **高性能**：基于 Rust 和 Axum 构建，速度极快。
-   📦 **GitHub 存储**：使用 GitHub 仓库作为存储，无限且免费。
-   ⚡ **CDN 加速**：自动生成 jsDelivr CDN 链接，实现全球快速访问。
-   📂 **文件支持**：支持上传图片（JPG, PNG, GIF, WEBP, HEIC）和其他文件（PDF, ZIP 等）。
-   🔒 **安全可靠**：支持 GitHub OAuth 和个人访问令牌 (PAT) 两种认证方式。
-   📱 **响应式界面**：精美的玻璃拟态 UI 设计，完美适配桌面和移动端。
-   🛠️ **自动配置**：需要时自动创建存储仓库和 GitHub Pages 分支。

## 安装

### 前置要求

-   Rust (最新稳定版)
-   GitHub 账号

### 源码编译

1.  克隆仓库：
    ```bash
    git clone https://github.com/yourusername/rustpic.git
    cd rustpic
    ```

2.  编译并运行：
    ```bash
    cargo run --release
    ```

3.  在浏览器中打开 `http://localhost:3002`。

## 配置

你可以通过环境变量或 `.env` 文件配置 RustPic。

| 变量名 | 描述 | 是否必须 |
| :--- | :--- | :--- |
| `GITHUB_CLIENT_ID` | GitHub OAuth Client ID | 否 (如果使用 PAT) |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth Client Secret | 否 (如果使用 PAT) |
| `OAUTH_CALLBACK_URL` | OAuth 回调地址 (默认: `http://localhost:3002/auth/callback`) | 否 |

## 使用方法

1.  **登录**：使用 "Continue with GitHub"（如果已配置）或输入你的 GitHub 个人访问令牌 (PAT)。
2.  **上传**：拖拽文件或点击选择。支持最大 50MB 的文件。
3.  **管理**：查看已上传的文件，复制 CDN 链接，或直接在仪表盘中删除文件。

## 许可证

MIT License
