<script setup lang="ts">
import { reactive, ref } from "vue";
import { ElMessage } from "element-plus";
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
  <section class="page-shell">
    <el-row :gutter="16">
      <el-col :xs="24" :lg="12">
        <el-card shadow="hover">
          <template #header>
            <div class="font-semibold">镜像与输出</div>
          </template>
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
        </el-card>
      </el-col>

      <el-col :xs="24" :lg="12">
        <el-card shadow="hover">
          <template #header>
            <div class="flex items-center justify-between">
              <span class="font-semibold">连接校验</span>
              <el-button size="small" :loading="testing" @click="handleTestAuth">测试连通与鉴权</el-button>
            </div>
          </template>
          <div class="space-y-2 text-sm leading-6 text-zinc-600">
            <p class="m-0">
              填写 Registry 与镜像名称后，可先测试连通性和鉴权是否通过，再执行导出。
              认证与代理参数来自「设置」页。
            </p>
            <p class="m-0">支持任意兼容 Registry V2 协议的仓库：Docker Hub、Harbor、Nexus 等。</p>
            <el-alert v-if="testMessage" :title="testMessage" type="success" :closable="false" />
          </div>
          <div class="mt-4 flex justify-end">
            <el-button type="primary" :loading="pulling" @click="handlePull">开始导出</el-button>
          </div>
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

    <input ref="directoryPicker" type="file" class="hidden" webkitdirectory directory @change="onDirectoryPicked" />
  </section>
</template>
