<script setup lang="ts">
import { onMounted, ref } from "vue";
import SearchPullView from "./components/SearchPullView.vue";
import DirectPullView from "./components/DirectPullView.vue";
import SettingsView from "./components/SettingsView.vue";
import { ensureConfigLoaded } from "./state/config";

const activeTab = ref("search");

onMounted(() => {
  void ensureConfigLoaded();
});
</script>

<template>
  <div class="mx-auto h-full max-w-6xl overflow-y-auto px-6 py-6">
    <header class="mb-5">
      <h1 class="m-0 text-2xl font-bold tracking-tight text-zinc-900">DockerHub Puller</h1>
      <p class="mt-1 text-sm text-zinc-500">
        搜索 / 拉取容器镜像并导出为 <code>docker load</code> 可导入的 Tar 包
      </p>
    </header>

    <el-tabs v-model="activeTab">
      <el-tab-pane label="镜像搜索" name="search">
        <SearchPullView />
      </el-tab-pane>
      <el-tab-pane label="直接拉取" name="direct">
        <DirectPullView />
      </el-tab-pane>
      <el-tab-pane label="设置" name="settings">
        <SettingsView />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>
