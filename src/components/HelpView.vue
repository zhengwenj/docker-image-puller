<script setup lang="ts">
import { Connection, Document, Guide, QuestionFilled } from "@element-plus/icons-vue";
</script>

<template>
  <section class="space-y-5 pb-8">
    <div class="grid items-start gap-5 lg:grid-cols-2">
      <!-- 快速上手 -->
      <section class="panel">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Guide /></el-icon></span>
          <h2 class="panel-title m-0">快速上手</h2>
        </header>
        <div class="p-5">
          <ol class="m-0 grid list-none gap-4 p-0">
            <li class="flex gap-3">
              <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand-soft text-xs font-bold text-brand">1</span>
              <div class="text-[13px] leading-6 text-ink-dim">
                在「镜像搜索」页输入关键字（如 <code class="font-mono text-ink">nginx</code>），找到镜像后点击「导出 Tar」。
              </div>
            </li>
            <li class="flex gap-3">
              <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand-soft text-xs font-bold text-brand">2</span>
              <div class="text-[13px] leading-6 text-ink-dim">
                选择 Tag 与输出目录，点击「开始导出」。耗时取决于镜像大小，完成后可直接打开所在目录。
              </div>
            </li>
            <li class="flex gap-3">
              <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand-soft text-xs font-bold text-brand">3</span>
              <div class="min-w-0 flex-1 text-[13px] leading-6 text-ink-dim">
                将 Tar 文件拷贝到目标机器，执行以下命令导入：
                <code class="mt-2 block break-all rounded-lg bg-abyss/60 px-3 py-2 font-mono text-xs text-ink">docker load -i nginx-latest.tar</code>
              </div>
            </li>
          </ol>
          <p class="m-0 mt-4 border-t border-edge pt-3 text-xs leading-5 text-ink-faint">
            导入后用 <code class="font-mono">docker images</code> 查看，<code class="font-mono">docker run</code> 正常使用。
          </p>
        </div>
      </section>

      <!-- 直接拉取与仓库 -->
      <section class="panel">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Document /></el-icon></span>
          <h2 class="panel-title m-0">直接拉取与仓库</h2>
        </header>
        <div class="space-y-3 p-5 text-[13px] leading-6 text-ink-dim">
          <p class="m-0">已知镜像全名时可用「直接拉取」页，无需搜索。镜像名称支持以下格式：</p>
          <div class="grid gap-1.5 rounded-xl bg-abyss/60 px-3.5 py-3 font-mono text-xs leading-5">
            <div class="text-ink">library/nginx</div>
            <div class="text-ink">username/my-app:1.2.0</div>
            <div class="text-ink">harbor.example.com/proj/app</div>
          </div>
          <p class="m-0">
            支持任意 Registry V2 兼容仓库（Docker Hub、Harbor、Nexus 等）。拉取私有仓库镜像时勾选「携带认证信息」，
            账号在「设置 → 网络与认证」中配置；Docker Hub 建议使用 Access Token 代替密码。
          </p>
          <p class="m-0">
            导出前请在「设置」中选择目标平台，默认 <code class="font-mono text-ink">linux/amd64</code>；
            Apple Silicon、树莓派等 ARM 设备请选 <code class="font-mono text-ink">arm64</code>。
          </p>
        </div>
      </section>

      <!-- 网络与代理 -->
      <section class="panel">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Connection /></el-icon></span>
          <h2 class="panel-title m-0">网络与代理</h2>
        </header>
        <div class="space-y-3 p-5 text-[13px] leading-6 text-ink-dim">
          <p class="m-0">默认直连。网络受限时在「设置 → 代理地址」填写代理，支持以下协议：</p>
          <div class="grid gap-1.5 rounded-xl bg-abyss/60 px-3.5 py-3 font-mono text-xs leading-5">
            <div class="text-ink">http://127.0.0.1:7890</div>
            <div class="text-ink">https://proxy.corp.example.com</div>
            <div class="text-ink">socks5://127.0.0.1:1080</div>
            <div class="text-ink">socks5h://127.0.0.1:1080 <span class="text-ink-faint">(域名由代理端解析)</span></div>
          </div>
          <p class="m-0">代理对镜像搜索与拉取的所有请求生效；留空则直连。保存设置后立即生效，无需重启。</p>
        </div>
      </section>

      <!-- 常见问题 -->
      <section class="panel">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><QuestionFilled /></el-icon></span>
          <h2 class="panel-title m-0">常见问题</h2>
        </header>
        <div class="grid gap-4 p-5">
          <div>
            <div class="text-[13px] font-semibold text-ink">提示鉴权失败（401）？</div>
            <p class="m-0 mt-1 text-[13px] leading-6 text-ink-dim">
              检查用户名和密码是否正确；Docker Hub 用户建议在官网生成 Access Token 填入密码栏。
            </p>
          </div>
          <div>
            <div class="text-[13px] font-semibold text-ink">提示 404 找不到镜像？</div>
            <p class="m-0 mt-1 text-[13px] leading-6 text-ink-dim">
              Tag 不存在或镜像名称拼写错误；私有仓库需确认账号有该仓库的拉取权限。
            </p>
          </div>
          <div>
            <div class="text-[13px] font-semibold text-ink">连接超时或代理报错？</div>
            <p class="m-0 mt-1 text-[13px] leading-6 text-ink-dim">
              确认代理软件已启动、地址端口正确、协议类型（http / socks5）选对；也可清空代理直连重试。
            </p>
          </div>
          <div>
            <div class="text-[13px] font-semibold text-ink">导出的 Tar 包含什么？</div>
            <p class="m-0 mt-1 text-[13px] leading-6 text-ink-dim">
              完整的 OCI 镜像层与 manifest，兼容 <code class="font-mono">docker load</code>，离线环境可永久保存复用。
            </p>
          </div>
        </div>
      </section>
    </div>
  </section>
</template>
