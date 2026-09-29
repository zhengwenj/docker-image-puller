<div align="center">

# 🐳 Docker Image Puller

**Pull container images from any Registry V2 and export offline, `docker load`-ready tar archives.**

搜索、拉取容器镜像并导出为离线 Tar 包的桌面工具，开箱即用。

![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![Built with](https://img.shields.io/badge/built%20with-Tauri%202%20%2B%20Vue%203-orange)

</div>

## 为什么需要它

在生产环境中经常遇到这样的场景：目标机器在**内网 / 离线机房**，无法直接 `docker pull`，而手工 `docker save` 又必须先有一台装好 Docker 且能联网的机器。

Docker Image Puller 解决这个问题：在**任意一台能联网的电脑**（无需安装 Docker）上，把镜像直接拉取并打包成标准的 `docker load` 可导入 Tar 文件，拷贝到目标机器一条命令完成导入。

```
能联网的电脑                           离线目标机器
┌─────────────────────┐              ┌──────────────────┐
│ Docker Image Puller │              │                  │
│  搜索 / 拉取 / 导出  │─── U盘/SCP ──▶│ docker load -i   │
│  → image.tar        │              │  docker run ...  │
└─────────────────────┘              └──────────────────┘
```

## ✨ 功能特性

- 🔍 **镜像搜索** — 通过 Docker Hub 公共搜索 API 检索镜像，展示星标、拉取次数、描述与官方标记
- 📦 **导出离线 Tar** — 生成兼容 `docker load` 的 OCI/Docker Tar 包（同时包含 `manifest.json`、`repositories`、`oci-layout`、`index.json`）
- 🌐 **任意 Registry V2 仓库** — 支持 Docker Hub、Harbor、Nexus 等所有兼容 Registry V2 协议的仓库
- 🔐 **私有仓库认证** — 用户名 + 密码或 Access Token 拉取私有镜像
- 🧩 **多架构平台选择** — linux/windows × amd64/arm64/arm/386/ppc64le/s390x，默认 `linux/amd64`
- ⚡ **并行下载与校验** — 镜像层按 CPU 核数 2~8 并发下载，逐层校验 sha256 digest
- 🛡 **代理支持** — `http` / `https` / `socks5` / `socks5h` 代理，对搜索与拉取请求统一生效
- 🌗 **亮色 / 暗色主题** — 深色开发者工具风格界面，主题选择自动记忆
- 💾 **配置持久化** — 输出目录、默认 Tag、平台、代理、账号本地保存，重启自动恢复

## 📥 安装

### 下载安装包

前往 [Releases](../../releases) 页面下载对应平台的安装包：

| 平台 | 文件 |
| --- | --- |
| Windows | `.msi` / `.exe`（需要 WebView2，Win10/11 一般已内置） |

### 从源码构建

需要安装：Rust（stable）、Node.js 18+ 与 pnpm，以及 Tauri 2 的系统依赖（Linux 需要 webkit2gtk 等）。

```bash
# 安装前端依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 打包
pnpm tauri build
```

## 🚀 快速上手

1. 在「**镜像搜索**」页输入关键字（如 `nginx`），点击结果旁的「导出 Tar」；
2. 选择 Tag 与输出目录，点击「开始导出」，等待完成；
3. 将 Tar 文件拷贝到目标机器，执行导入：

```bash
docker load -i nginx-latest.tar
docker images   # 查看导入结果
```

已知镜像名时也可以直接在「**直接拉取**」页输入 `library/nginx`、`harbor.example.com/proj/app` 等地址拉取，支持先测试连通性与鉴权。

**代理配置**：在「设置 → 代理地址」填写即可，支持以下格式：

```
http://127.0.0.1:7890
https://proxy.corp.example.com
socks5://127.0.0.1:1080
socks5h://127.0.0.1:1080   # 域名由代理端解析
```

更多说明见应用内「帮助」页。

## ❓ 常见问题

<details>
<summary><b>提示鉴权失败（401）？</b></summary>

检查用户名和密码；Docker Hub 用户建议使用 [Access Token](https://app.docker.com/settings/personal-access-tokens) 代替账号密码。
</details>

<details>
<summary><b>提示 404 找不到镜像？</b></summary>

Tag 不存在或镜像名称拼写错误；私有仓库需确认账号有对应仓库的拉取权限。
</details>

<details>
<summary><b>连接超时或代理报错？</b></summary>

确认代理软件已启动、地址端口正确、协议类型（http / socks5）选择正确；也可以清空代理直连重试。
</details>

<details>
<summary><b>与 skopeo / crane 有什么区别？</b></summary>

[skopeo](https://github.com/containers/skopeo)、[crane](https://github.com/google/go-containerregistry) 是优秀的命令行工具，同样可以把镜像保存为 tar。Docker Image Puller 的定位是**图形界面、开箱即用**：内置搜索、目录选择、代理与认证配置，适合不常使用命令行的场景。
</details>

## 🧰 技术栈

Rust + Tauri 2 · Vue 3 + TypeScript · Element Plus + Tailwind CSS · Vite

```
├── src/                  # Vue 前端
│   ├── components/       # 搜索拉取 / 直接拉取 / 设置 / 帮助 页面
│   ├── composables/      # Tauri invoke 封装
│   ├── state/            # 共享配置状态
│   └── types/            # 类型定义
└── src-tauri/            # Rust 后端
    └── src/lib.rs        # Registry V2 拉取、镜像层下载、Tar 组装
```

## ⚠️ 已知限制

- 搜索功能使用 Docker Hub 公共搜索 API，私有仓库不会出现在搜索结果中（可直接使用「直接拉取」页签）。
- 镜像层下载依赖 Registry V2 HTTP API，若仓库禁用了该协议则无法使用。

## 📄 License

[MIT](LICENSE)
