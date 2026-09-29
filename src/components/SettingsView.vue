<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { Cpu, Lock } from "@element-plus/icons-vue";
import { appConfig, persistConfig, resolvePickedDirectory } from "../state/config";

const saving = ref(false);
const directoryPicker = ref<HTMLInputElement | null>(null);

function openDirectoryPicker() {
  directoryPicker.value?.click();
}

function onDirectoryPicked(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0] as (File & { path?: string; webkitRelativePath?: string }) | undefined;
  const selected = file ? resolvePickedDirectory(file) : "";
  if (selected) appConfig.outputDir = selected;
  input.value = "";
}

async function handleSave() {
  saving.value = true;
  try {
    await persistConfig();
    ElMessage.success("配置已保存");
  } catch (err) {
    ElMessage.error(String(err));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <section class="space-y-5 pb-8">
    <div class="grid items-start gap-5 lg:grid-cols-2">
      <!-- 导出默认值 -->
      <section class="panel">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Cpu /></el-icon></span>
          <h2 class="panel-title m-0">导出默认值</h2>
        </header>
        <div class="p-5">
          <el-form label-position="top">
            <el-form-item label="默认输出目录">
              <div class="flex w-full gap-2">
                <el-input v-model="appConfig.outputDir" placeholder="如 E:/docker-images" />
                <el-button @click="openDirectoryPicker">选择目录</el-button>
              </div>
            </el-form-item>
            <el-form-item label="默认 Tag">
              <el-input v-model="appConfig.defaultTag" placeholder="latest" />
            </el-form-item>
            <div class="grid grid-cols-2 gap-4">
              <el-form-item label="目标 OS">
                <el-select v-model="appConfig.platformOs">
                  <el-option label="linux" value="linux" />
                  <el-option label="windows" value="windows" />
                </el-select>
              </el-form-item>
              <el-form-item label="目标架构">
                <el-select v-model="appConfig.platformArchitecture">
                  <el-option label="amd64" value="amd64" />
                  <el-option label="arm64" value="arm64" />
                  <el-option label="arm" value="arm" />
                  <el-option label="386" value="386" />
                  <el-option label="ppc64le" value="ppc64le" />
                  <el-option label="s390x" value="s390x" />
                </el-select>
              </el-form-item>
            </div>
          </el-form>
        </div>
      </section>

      <!-- 网络与认证 -->
      <section class="panel">
        <header class="panel-head">
          <span class="panel-icon"><el-icon :size="15"><Lock /></el-icon></span>
          <h2 class="panel-title m-0">网络与认证</h2>
        </header>
        <div class="p-5">
          <el-form label-position="top">
            <div class="mb-1 text-xs font-semibold uppercase tracking-wider text-ink-faint">代理</div>
            <el-form-item label="代理地址（可选）">
              <el-input v-model="appConfig.proxy" clearable placeholder="http://127.0.0.1:7890 或 socks5://127.0.0.1:1080" />
            </el-form-item>
            <p class="m-0 text-xs leading-5 text-ink-faint">
              支持 http、https、socks5、socks5h 协议，对搜索与镜像拉取的所有请求生效；留空则直连。
            </p>

            <div class="mb-1 mt-4 text-xs font-semibold uppercase tracking-wider text-ink-faint">
              认证（Docker Hub / 私有仓库）
            </div>
            <el-form-item label="用户名">
              <el-input v-model="appConfig.username" placeholder="仓库账号，留空则匿名访问" />
            </el-form-item>
            <el-form-item label="密码 / Access Token">
              <el-input v-model="appConfig.password" type="password" show-password placeholder="密码或长期 Token" />
            </el-form-item>
          </el-form>
        </div>
      </section>
    </div>

    <!-- 保存栏 -->
    <div class="flex items-center justify-between rounded-2xl border border-edge bg-panel px-5 py-3.5">
      <span class="text-xs text-ink-faint">配置保存在本地，导出时自动携带</span>
      <el-button type="primary" :loading="saving" @click="handleSave">保存配置</el-button>
    </div>

    <input ref="directoryPicker" type="file" class="hidden" webkitdirectory directory @change="onDirectoryPicked" />
  </section>
</template>
