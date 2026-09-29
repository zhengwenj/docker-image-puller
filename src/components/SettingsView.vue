<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
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
  <section class="page-shell">
    <el-row :gutter="16">
      <el-col :xs="24" :lg="12">
        <el-card shadow="hover">
          <template #header>
            <div class="font-semibold">导出默认值</div>
          </template>
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
            <el-row :gutter="10">
              <el-col :span="12">
                <el-form-item label="目标 OS">
                  <el-select v-model="appConfig.platformOs">
                    <el-option label="linux" value="linux" />
                    <el-option label="windows" value="windows" />
                  </el-select>
                </el-form-item>
              </el-col>
              <el-col :span="12">
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
              </el-col>
            </el-row>
          </el-form>
        </el-card>
      </el-col>

      <el-col :xs="24" :lg="12">
        <el-card shadow="hover">
          <template #header>
            <div class="font-semibold">网络与认证</div>
          </template>
          <el-form label-position="top">
            <el-divider class="!mt-0">代理</el-divider>
            <el-form-item label="HTTP_PROXY">
              <el-input v-model="appConfig.httpProxy" placeholder="http://127.0.0.1:7890" />
            </el-form-item>
            <el-form-item label="HTTPS_PROXY">
              <el-input v-model="appConfig.httpsProxy" placeholder="http://127.0.0.1:7890" />
            </el-form-item>
            <el-form-item label="NO_PROXY">
              <el-input v-model="appConfig.noProxy" placeholder="localhost,.corp.local" />
            </el-form-item>

            <el-divider>认证（Docker Hub / 私有仓库）</el-divider>
            <el-form-item label="用户名">
              <el-input v-model="appConfig.username" placeholder="仓库账号，留空则匿名访问" />
            </el-form-item>
            <el-form-item label="密码 / Access Token">
              <el-input v-model="appConfig.password" type="password" show-password placeholder="密码或长期 Token" />
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
    </el-row>

    <div class="flex justify-end">
      <el-button type="primary" :loading="saving" @click="handleSave">保存配置</el-button>
    </div>

    <input ref="directoryPicker" type="file" class="hidden" webkitdirectory directory @change="onDirectoryPicked" />
  </section>
</template>
