<script setup lang="ts">
import { ref, reactive } from "vue";
import { ElMessage } from "element-plus";
import { Download, Search, Star } from "@element-plus/icons-vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { pullImageAsTar, searchImages } from "../composables/useRegistryApi";
import type { SearchImageResult } from "../types/registry";
import {
  appConfig,
  authConfigOrUndefined,
  formatCount,
  formatTime,
  persistConfig,
  proxyConfigOrUndefined,
  resolvePickedDirectory,
} from "../state/config";

const keyword = ref("");
const searching = ref(false);
const pulling = ref(false);
const error = ref("");
const progressMessage = ref("");
const results = ref<SearchImageResult[]>([]);
const hasSearched = ref(false);
const selectedItem = ref<SearchImageResult | null>(null);
const pullDialogVisible = ref(false);
const pullForm = reactive({
  tag: "latest",
  outputDir: "",
  tarFileName: "",
});

const hotWords = ["nginx", "redis", "postgres", "node", "mysql"];

const directoryPicker = ref<HTMLInputElement | null>(null);

interface ExportResult {
  imageRef: string;
  tarPath: string;
  finishedAt: string;
}

const exportResult = ref<ExportResult | null>(null);

async function handleSearch() {
  const value = keyword.value.trim();
  if (!value) {
    results.value = [];
    hasSearched.value = false;
    return;
  }
  error.value = "";
  searching.value = true;
  hasSearched.value = true;
  try {
    results.value = await searchImages(value, proxyConfigOrUndefined(), 30);
  } catch (err) {
    error.value = String(err);
  } finally {
    searching.value = false;
  }
}

function quickSearch(word: string) {
  keyword.value = word;
  void handleSearch();
}

function openPullDialog(item: SearchImageResult) {
  selectedItem.value = item;
  pullForm.tag = appConfig.defaultTag || "latest";
  pullForm.outputDir = appConfig.outputDir;
  pullForm.tarFileName = "";
  pullDialogVisible.value = true;
}

function openDirectoryPicker() {
  directoryPicker.value?.click();
}

function onDirectoryPicked(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0] as (File & { path?: string; webkitRelativePath?: string }) | undefined;
  const selected = file ? resolvePickedDirectory(file) : "";
  if (selected) pullForm.outputDir = selected;
  input.value = "";
}

async function startPull() {
  const item = selectedItem.value;
  if (!item) return;
  const outputDir = pullForm.outputDir.trim();
  if (!outputDir) {
    error.value = "请先填写输出目录";
    return;
  }

  error.value = "";
  exportResult.value = null;
  progressMessage.value = "正在拉取镜像并导出 Tar（耗时取决于镜像大小）...";
  pulling.value = true;
  try {
    appConfig.outputDir = outputDir;
    await persistConfig();
    const result = await pullImageAsTar({
      registry: "docker.io",
      repository: item.fullName,
      tag: pullForm.tag.trim() || "latest",
      platformOs: appConfig.platformOs.trim() || "linux",
      platformArchitecture: appConfig.platformArchitecture.trim() || "amd64",
      outputDir,
      tarFileName: pullForm.tarFileName.trim() || undefined,
      proxy: proxyConfigOrUndefined(),
      auth: authConfigOrUndefined(),
    });
    progressMessage.value = "";
    exportResult.value = {
      imageRef: result.imageRef,
      tarPath: result.tarPath,
      finishedAt: formatTime(new Date()),
    };
    ElMessage.success("镜像导出完成");
    pullDialogVisible.value = false;
  } catch (err) {
    progressMessage.value = "";
    error.value = String(err);
  } finally {
    pulling.value = false;
  }
}

async function openExportDirectory() {
  if (!exportResult.value) return;
  try {
    await revealItemInDir(exportResult.value.tarPath);
  } catch {
    ElMessage.error("打开目录失败");
  }
}
</script>

<template>
  <section class="space-y-5 pb-8">
    <!-- 搜索区 -->
    <section class="panel p-5">
      <div class="flex gap-3">
        <el-input
          v-model="keyword"
          size="large"
          placeholder="输入镜像关键字，如 nginx、redis、postgres"
          clearable
          @keyup.enter="handleSearch"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button size="large" type="primary" class="px-7" :loading="searching" @click="handleSearch">
          搜索
        </el-button>
      </div>
      <div class="mt-3.5 flex flex-wrap items-center gap-2 text-xs text-ink-faint">
        <span>热门镜像</span>
        <button v-for="word in hotWords" :key="word" type="button" class="hot-chip" @click="quickSearch(word)">
          {{ word }}
        </button>
      </div>
    </section>

    <!-- 搜索结果 -->
    <section>
      <div class="mb-3 flex items-center justify-between">
        <h2 class="m-0 text-sm font-semibold text-ink">搜索结果</h2>
        <span v-if="results.length" class="text-xs text-ink-faint">共 {{ results.length }} 条</span>
      </div>

      <!-- 加载骨架 -->
      <div v-if="searching" class="grid gap-2.5">
        <div v-for="i in 4" :key="i" class="result-card opacity-70">
          <el-skeleton animated style="flex: 1" :rows="2" />
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else-if="!results.length" class="panel flex flex-col items-center justify-center py-14">
        <el-icon :size="40" class="text-ink-faint"><Search /></el-icon>
        <p class="m-0 mt-3 text-sm text-ink-dim">{{ hasSearched ? "没有找到相关镜像，换个关键字试试" : "输入关键字开始搜索镜像" }}</p>
      </div>

      <!-- 结果卡片列表 -->
      <div v-else class="grid gap-2.5">
        <article v-for="row in results" :key="row.fullName" class="result-card">
          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-2">
              <span class="result-name">{{ row.fullName }}</span>
              <span v-if="row.isOfficial" class="tag-official">OFFICIAL</span>
            </div>
            <p class="m-0 mt-1 line-clamp-2 text-[13px] leading-5 text-ink-dim">
              {{ row.description || "无描述" }}
            </p>
            <div class="mt-2 flex items-center gap-1.5">
              <span class="meta-chip"><el-icon :size="11"><Star /></el-icon>{{ formatCount(row.stars) }}</span>
              <span class="meta-chip"><el-icon :size="11"><Download /></el-icon>{{ formatCount(row.pulls) }} pulls</span>
            </div>
          </div>
          <el-button type="primary" plain class="shrink-0" @click="openPullDialog(row)">导出 Tar</el-button>
        </article>
      </div>
    </section>

    <!-- 状态区 -->
    <div class="grid gap-2.5">
      <el-alert v-if="progressMessage" :title="progressMessage" type="info" :closable="false" show-icon />
      <el-alert v-if="error" :title="error" type="error" :closable="false" />

      <div v-if="exportResult" class="success-panel">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div class="flex items-center gap-3">
            <el-icon :size="22" color="#22C55E"><Download /></el-icon>
            <div>
              <p class="m-0 text-[15px] font-bold text-ink">导出成功</p>
              <p class="m-0 mt-0.5 text-xs text-ink-faint">完成时间：{{ exportResult.finishedAt }}</p>
            </div>
          </div>
          <el-button type="primary" @click="openExportDirectory">打开目录</el-button>
        </div>
        <div class="mt-3 grid gap-1.5 rounded-xl bg-abyss/50 px-3.5 py-3 text-xs leading-5">
          <div class="flex gap-2">
            <span class="w-10 shrink-0 text-ink-faint">镜像</span>
            <code class="font-mono text-ink-dim">{{ exportResult.imageRef }}</code>
          </div>
          <div class="flex gap-2">
            <span class="w-10 shrink-0 text-ink-faint">文件</span>
            <code class="break-all font-mono text-ink-dim">{{ exportResult.tarPath }}</code>
          </div>
        </div>
      </div>
    </div>

    <!-- 导出对话框 -->
    <el-dialog v-model="pullDialogVisible" title="导出镜像 Tar" width="620px">
      <el-form label-position="top">
        <el-form-item label="镜像">
          <el-input :value="selectedItem?.fullName || ''" readonly />
        </el-form-item>
        <el-form-item label="Tag">
          <el-input v-model="pullForm.tag" placeholder="latest" />
        </el-form-item>
        <el-form-item label="输出目录">
          <div class="flex w-full gap-2">
            <el-input v-model="pullForm.outputDir" placeholder="选择或输入输出目录" />
            <el-button @click="openDirectoryPicker">选择目录</el-button>
          </div>
        </el-form-item>
        <el-form-item label="Tar 文件名（可选）">
          <el-input v-model="pullForm.tarFileName" placeholder="留空自动生成" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="pullDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="pulling" @click="startPull">开始导出</el-button>
      </template>
    </el-dialog>

    <input ref="directoryPicker" type="file" class="hidden" webkitdirectory directory @change="onDirectoryPicked" />
  </section>
</template>
