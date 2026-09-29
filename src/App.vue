<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { Box, Download, Link, Moon, QuestionFilled, Search, Setting, Sunny } from "@element-plus/icons-vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import SearchPullView from "./components/SearchPullView.vue";
import DirectPullView from "./components/DirectPullView.vue";
import SettingsView from "./components/SettingsView.vue";
import HelpView from "./components/HelpView.vue";
import { ensureConfigLoaded } from "./state/config";
import { theme, toggleTheme } from "./state/theme";

const REPO_URL = "https://github.com/zhengwenj/docker-image-puller";
const REPO_PATH = "zhengwenj/docker-image-puller";

const activeTab = ref("search");
const appVersion = ref("1.0.0");

function openRepo() {
  openUrl(REPO_URL).catch(() => window.open(REPO_URL, "_blank"));
}

const pages = [
  {
    key: "search",
    label: "镜像搜索",
    title: "镜像搜索",
    desc: "从 Docker Hub 检索镜像，拉取并导出为 docker load 可导入的 Tar 包",
    icon: Search,
    component: SearchPullView,
  },
  {
    key: "direct",
    label: "直接拉取",
    title: "直接拉取",
    desc: "指定 Registry 地址与镜像名，直连任意 Registry V2 兼容仓库导出 Tar 包",
    icon: Download,
    component: DirectPullView,
  },
  {
    key: "settings",
    label: "设置",
    title: "设置",
    desc: "导出默认值、代理与仓库认证配置",
    icon: Setting,
    component: SettingsView,
  },
  {
    key: "help",
    label: "帮助",
    title: "帮助",
    desc: "使用说明、代理配置与常见问题",
    icon: QuestionFilled,
    component: HelpView,
  },
] as const;

onMounted(() => {
  void ensureConfigLoaded();
  getVersion()
    .then((version) => {
      if (version) appVersion.value = version;
    })
    .catch(() => {
      // 浏览器预览模式取不到版本号,保留默认值
    });
});
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <!-- 品牌区 -->
      <div class="flex items-center gap-3 px-5 pb-6 pt-6">
        <div class="brand-badge">
          <el-icon :size="22" color="#fff"><Box /></el-icon>
        </div>
        <div class="leading-tight">
          <div class="text-[15px] font-bold tracking-tight text-ink">Docker Image Puller</div>
          <div class="mt-0.5 text-[11px] text-ink-faint">镜像拉取 · Tar 导出</div>
        </div>
      </div>

      <!-- 导航 -->
      <nav class="flex flex-col gap-1 px-3">
        <button
          v-for="page in pages"
          :key="page.key"
          type="button"
          class="nav-item"
          :class="{ 'is-active': activeTab === page.key }"
          @click="activeTab = page.key"
        >
          <el-icon :size="16"><component :is="page.icon" /></el-icon>
          <span>{{ page.label }}</span>
        </button>
      </nav>

      <!-- 底部状态 -->
      <div class="mt-auto px-5 pb-5">
        <div class="rounded-xl border border-edge bg-chip/[0.03] px-3 py-2.5 text-[11px] leading-5 text-ink-faint">
          <div class="flex items-center gap-1.5">
            <span class="h-1.5 w-1.5 rounded-full bg-mint"></span>
            Registry V2 兼容
          </div>
          <div class="mt-0.5">v{{ appVersion }}</div>
          <a
            :href="REPO_URL"
            class="mt-1 flex w-full min-w-0 cursor-pointer items-center gap-1.5 text-left text-[11px] leading-5 text-brand underline decoration-brand/40 underline-offset-2 transition-colors duration-150 hover:text-brand-deep hover:decoration-brand-deep"
            :title="REPO_URL"
            @click.prevent="openRepo"
          >
            <el-icon :size="11" class="shrink-0"><Link /></el-icon>
            <span class="truncate">{{ REPO_PATH }}</span>
          </a>
        </div>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="min-w-0 flex-1 overflow-y-auto">
      <div class="mx-auto max-w-5xl px-8 py-7">
        <Transition name="page-fade" mode="out-in">
          <div :key="activeTab" class="page-head">
            <div>
              <h1 class="page-title">{{ pages.find((p) => p.key === activeTab)?.title }}</h1>
              <p class="page-desc">{{ pages.find((p) => p.key === activeTab)?.desc }}</p>
            </div>
            <button
              type="button"
              class="icon-btn shrink-0"
              :title="theme === 'dark' ? '切换到亮色模式' : '切换到暗色模式'"
              :aria-label="theme === 'dark' ? '切换到亮色模式' : '切换到暗色模式'"
              @click="toggleTheme"
            >
              <el-icon :size="17">
                <Sunny v-if="theme === 'dark'" />
                <Moon v-else />
              </el-icon>
            </button>
          </div>
        </Transition>

        <Transition name="page-fade" mode="out-in">
          <component :is="pages.find((p) => p.key === activeTab)?.component" :key="activeTab" />
        </Transition>
      </div>
    </main>
  </div>
</template>
