<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { ElMessage } from "element-plus";
import { Box, CircleCheck, Connection, Download } from "@element-plus/icons-vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { pullImageAsTar, testRegistryAuth } from "../composables/useRegistryApi";
import {
  appConfig,
  authConfigOrUndefined,
  formatTime,
  persistConfig,
  proxyConfigOrUndefined,
  resolvePickedDirectory,
} from "../state/config";

const form = reactive({
  registry: "docker.io",
  repository: "",
  tag: "",
  outputDir: "",
  tarFileName: "",
  useAuth: false,
});

const directoryPicker = ref<HTMLInputElement | null>(null);
const pulling = ref(false);
const testing = ref(false);
const error = ref("");
const progressMessage = ref("");
const testMessage = ref("");

interface ExportResult {
  imageRef: string;
  tarPath: string;
  finishedAt: string;
}

const exportResult = ref<ExportResult | null>(null);

onMounted(() => {
  if (!form.outputDir && appConfig.outputDir) form.outputDir = appConfig.outputDir;
});

function openDirectoryPicker() {
  directoryPicker.value?.click();
}

function onDirectoryPicked(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0] as (File & { path?: string; webkitRelativePath?: string }) | undefined;
  const selected = file ? resolvePickedDirectory(file) : "";
  if (selected) form.outputDir = selected;
  input.value = "";
}

async function handleTestAuth() {
  error.value = "";
  testMessage.value = "";
  testing.value = true;
  try {
    const result = await testRegistryAuth({
      registry: form.registry.trim() || "docker.io",
      repository: form.repository.trim() || undefined,
      tag: form.tag.trim() || undefined,
      proxy: proxyConfigOrUndefined(),
      auth: authConfigOrUndefined(),
    });
    testMessage.value = result.message;
    ElMessage.success("校验通过");
  } catch (err) {
    error.value = String(err);
  } finally {
    testing.value = false;
  }
}

async function handlePull() {
  if (!form.repository.trim()) {
    error.value = "请填写镜像名称";
    return;
  }
  if (!form.outputDir.trim()) {
    error.value = "请填写输出目录";
    return;
  }

  error.value = "";
  exportResult.value = null;
  testMessage.value = "";
  progressMessage.value = "正在拉取镜像并导出 Tar（耗时取决于镜像大小）...";
  pulling.value = true;
  try {
    await persistConfig();
    const result = await pullImageAsTar({
      registry: form.registry.trim() || "docker.io",
      repository: form.repository.trim(),
      tag: form.tag.trim() || undefined,
      platformOs: appConfig.platformOs.trim() || "linux",
      platformArchitecture: appConfig.platformArchitecture.trim() || "amd64",
      outputDir: form.outputDir.trim(),
      tarFileName: form.tarFileName.trim() || undefined,
      proxy: proxyConfigOrUndefined(),
      auth: form.useAuth ? authConfigOrUndefined() : undefined,
    });
    progressMessage.value = "";
    exportResult.value = {
      imageRef: result.imageRef,
      tarPath: result.tarPath,
      finishedAt: formatTime(new Date()),
    };
    ElMessage.success("导出完成");
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
    <div class="grid items-start gap-5 lg:grid-cols-5">
      <!-- 镜像与输出 -->
      <section class="panel lg:col-span-3">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Box /></el-icon></span>
          <h2 class="panel-title m-0">镜像与输出</h2>
        </header>
        <div class="p-5">
          <el-form label-position="top">
            <el-form-item label="Registry 地址">
              <el-input v-model="form.registry" placeholder="docker.io（默认）或私有仓库地址" />
            </el-form-item>
            <el-form-item label="镜像名称">
              <el-input v-model="form.repository" placeholder="library/nginx 或 harbor.example.com/proj/app" />
            </el-form-item>
            <el-form-item label="Tag">
              <el-input v-model="form.tag" :placeholder="appConfig.defaultTag || 'latest'" />
            </el-form-item>
            <el-form-item label="输出目录">
              <div class="flex w-full gap-2">
                <el-input v-model="form.outputDir" placeholder="选择或输入输出目录" />
                <el-button @click="openDirectoryPicker">选择目录</el-button>
              </div>
            </el-form-item>
            <el-form-item label="Tar 文件名（可选）">
              <el-input v-model="form.tarFileName" placeholder="留空自动生成" />
            </el-form-item>
            <el-form-item>
              <el-checkbox v-model="form.useAuth">携带认证信息（拉取私有仓库镜像时勾选）</el-checkbox>
            </el-form-item>
          </el-form>
          <div class="flex justify-end border-t border-edge pt-4">
            <el-button type="primary" size="large" class="px-8" :loading="pulling" @click="handlePull">
              开始导出
            </el-button>
          </div>
        </div>
      </section>

      <!-- 连接校验 -->
      <section class="panel self-stretch lg:col-span-2">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Connection /></el-icon></span>
          <h2 class="panel-title m-0">连接校验</h2>
        </header>
        <div class="flex h-[calc(100%-57px)] flex-col p-5">
          <div class="space-y-2.5 text-[13px] leading-6 text-ink-dim">
            <p class="m-0">填写 Registry 与镜像名称后，可先测试连通性和鉴权是否通过，再执行导出。认证与代理参数来自「设置」页。</p>
            <p class="m-0">支持任意兼容 Registry V2 协议的仓库：Docker Hub、Harbor、Nexus 等。</p>
          </div>

          <el-button class="mt-4" :loading="testing" @click="handleTestAuth">测试连通与鉴权</el-button>

          <el-alert
            v-if="testMessage"
            class="mt-4"
            :title="testMessage"
            type="success"
            :closable="false"
          />

          <div class="mt-auto flex items-center gap-2 border-t border-edge pt-4 text-xs text-ink-faint">
            <el-icon :size="13" color="#22C55E"><CircleCheck /></el-icon>
            平台：{{ appConfig.platformOs || "linux" }}/{{ appConfig.platformArchitecture || "amd64" }}，可在「设置」中修改
          </div>
        </div>
      </section>
    </div>

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

    <input ref="directoryPicker" type="file" class="hidden" webkitdirectory directory @change="onDirectoryPicked" />
  </section>
</template>
