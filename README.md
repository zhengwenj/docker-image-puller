# DockerHub Puller

一个用于**搜索、拉取容器镜像并导出为 `docker load` 可导入 Tar 包**的桌面工具。

适合在不方便直接使用 Docker daemon 拉取镜像的环境中（如内网、离线机房），把镜像保存为离线 Tar 文件，再复制到目标机器执行 `docker load -i image.tar`。

## 功能特性

- 通过 Docker Hub 公共搜索 API 搜索镜像，展示星标、拉取次数和描述。
- 拉取镜像并导出为兼容 `docker load` 的 OCI/Docker Tar 包（同时包含 `manifest.json`、`repositories`、`oci-layout`、`index.json`）。
- 支持多架构镜像平台选择（linux/windows × amd64/arm64/arm/386 等），默认 `linux/amd64`。
- 支持任意兼容 Registry V2 协议的仓库：Docker Hub、Harbor、Nexus 等。
- 支持 Docker Hub 用户名 + 密码或 Access Token 认证拉取私有镜像。
- 支持 HTTP_PROXY / HTTPS_PROXY / NO_PROXY 代理配置。
- 镜像层并行下载（按 CPU 核数 2~8 并发），下载后校验 sha256 digest。
- 配置（输出目录、默认 tag、平台、代理、账号）本地持久化，重启后自动恢复。

## 界面说明

| 页签 | 说明 |
| --- | --- |
| 镜像搜索 | 关键字搜索 Docker Hub 镜像，选择结果后填写 tag 与输出目录导出 Tar |
| 直接拉取 | 直接输入 Registry + 镜像 + Tag 拉取导出，支持连接与鉴权测试、私有仓库认证 |
| 设置 | 输出目录、默认 tag、目标平台、代理、认证账号 |

## 使用示例

导出完成后，在目标机器导入镜像：

```bash
docker load -i nginx-latest.tar
docker images
```

## 开发环境

需要安装：

- Rust（stable）
- Node.js 18+ 与 pnpm（或 npm）
- Tauri 2 所需的系统依赖（Windows 需要 WebView2，Linux 需要 webkit2gtk 等）

```bash
# 安装前端依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 打包
pnpm tauri build
```

## 技术栈

- Rust + Tauri 2
- Vue 3 + TypeScript
- Element Plus + Tailwind CSS
- Vite

## 项目结构

```
├── src/                  # Vue 前端
│   ├── components/       # 搜索拉取 / 直接拉取 / 设置 三个页面
│   ├── composables/      # Tauri invoke 封装
│   ├── state/            # 共享配置状态
│   └── types/            # 类型定义
└── src-tauri/            # Rust 后端
    └── src/lib.rs        # Registry V2 拉取、镜像层下载、Tar 组装
```

## 已知限制

- 搜索功能使用 Docker Hub 公共搜索 API，私有仓库不会出现在搜索结果中（可直接使用「直接拉取」页签）。
- 镜像层下载依赖 Registry V2 HTTP API，若仓库禁用了该协议则无法使用。

## License

[MIT](./LICENSE)
