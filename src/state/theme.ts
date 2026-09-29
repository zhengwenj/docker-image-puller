import { ref, watchEffect } from "vue";

export type ThemeMode = "light" | "dark";

const STORAGE_KEY = "dhp-theme";

function initialTheme(): ThemeMode {
  try {
    return localStorage.getItem(STORAGE_KEY) === "dark" ? "dark" : "light";
  } catch {
    return "light";
  }
}

export const theme = ref<ThemeMode>(initialTheme());

watchEffect(() => {
  document.documentElement.classList.toggle("dark", theme.value === "dark");
  try {
    localStorage.setItem(STORAGE_KEY, theme.value);
  } catch {
    // localStorage 不可用时仅当前会话生效
  }
});

export function toggleTheme() {
  theme.value = theme.value === "dark" ? "light" : "dark";
}
