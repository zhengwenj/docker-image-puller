<script setup lang="ts">
import { ref, reactive } from "vue";
import { ElMessage } from "element-plus";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { pullImageAsTar, searchImages } from "../composables/useRegistryApi";
import type { SearchImageResult } from "../types/registry";
import {
  appConfig,
  authConfigOrUndefined,
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
const selectedItem = ref<SearchImageResult | null>(null);
const pullDialogVisible = ref(false);
const pullForm = reactive({
  tag: "latest",
  outputDir: "",
  tarFileName: "",
});

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
    return;
  }
  error.value = "";
  searching.value = true;
  try {
    results.value = await searchImages(value, proxyConfigOrUndefined(), 30);
  } catch (err) {
    error.value = String(err);
  } finally {
    searching.value = false;
  }
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
  <section class="page-shell">
    <el-row :gutter="16">
      <el-col :xs="24" :lg="9">
        <el-card shadow="hover">
          <template #header>
            <div class="font-semibold">搜索镜像</div>
          </template>

          <el-form label-position="top">
            <el-form-item label="关键字">
              <el-input
                v-model="keyword"
                placeholder="输入关键字，如 nginx, redis"
                clearable
                @keyup.enter="handleSearch"
              />
            </el-form-item>

            <div class="text-xs leading-6 text-zinc-500">
              搜索使用 Docker Hub 公共接口，代理与账号请在「设置」中配置。
            </div>

            <div class="mt-3 flex justify-end">
              <el-button type="primary" :loading="searching" @click="handleSearch">搜索</el-button>
            </div>
          </el-form>
        </el-card>
      </el-col>

      <el-col :xs="24" :lg="15">
        <el-card shadow="hover">
          <template #header>
            <div class="flex items-center justify-between">
              <span class="font-semibold">搜索结果</span>
              <span class="text-xs text-zinc-500">共 {{ results.length }} 条</span>
            </div>
          </template>

          <el-empty v-if="!results.length && !searching" description="还没有搜索结果" />

          <el-table v-else :data="results" stripe max-height="480">
            <el-table-column label="镜像名" min-width="200">
              <template #default="{ row }">
                <div class="font-medium">{{ row.fullName }}</div>
                <div class="text-xs text-zinc-500">★ {{ row.stars }} · Pulls {{ row.pulls }}</div>
              </template>
            </el-table-column>
            <el-table-column label="描述" min-width="240">
              <template #default="{ row }">
                <div class="text-sm text-zinc-600">{{ row.description || "无描述" }}</div>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="110" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" text @click="openPullDialog(row)">导出 Tar</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>

    <div class="status-stack">
      <el-alert v-if="progressMessage" :title="progressMessage" type="info" :closable="false" show-icon />
      <el-alert v-if="error" :title="error" type="error" :closable="false" />

      <el-card v-if="exportResult" shadow="never" class="border-blue-200 bg-blue-50/50">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="m-0 text-base font-bold text-zinc-900">导出成功</p>
            <p class="m-0 mt-0.5 text-xs text-zinc-500">完成时间：{{ exportResult.finishedAt }}</p>
          </div>
          <el-button @click="openExportDirectory">打开目录</el-button>
        </div>
        <div class="mt-2 grid gap-1">
          <div>
            <span class="mr-2 text-xs font-semibold uppercase tracking-wide text-zinc-500">镜像</span>
            <code>{{ exportResult.imageRef }}</code>
          </div>
          <div>
            <span class="mr-2 text-xs font-semibold uppercase tracking-wide text-zinc-500">文件</span>
            <code class="break-all">{{ exportResult.tarPath }}</code>
          </div>
        </div>
      </el-card>
    </div>

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
