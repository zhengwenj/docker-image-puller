/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // 双主题色板:具体值由 style.css 中的 --c-* CSS 变量按主题提供(RGB 三元组)
        abyss: "rgb(var(--c-abyss) / <alpha-value>)",
        side: "rgb(var(--c-side) / <alpha-value>)",
        panel: {
          DEFAULT: "rgb(var(--c-panel) / <alpha-value>)",
          hover: "rgb(var(--c-panel-hover) / <alpha-value>)",
        },
        edge: {
          DEFAULT: "rgb(var(--c-edge) / <alpha-value>)",
          strong: "rgb(var(--c-edge-strong) / <alpha-value>)",
        },
        ink: {
          DEFAULT: "rgb(var(--c-ink) / <alpha-value>)",
          dim: "rgb(var(--c-ink-dim) / <alpha-value>)",
          faint: "rgb(var(--c-ink-faint) / <alpha-value>)",
        },
        chip: "rgb(var(--c-chip) / <alpha-value>)",
        brand: {
          DEFAULT: "#2496ED",
          deep: "#1B7FD4",
          soft: "rgba(36, 150, 237, 0.14)",
        },
        mint: {
          DEFAULT: "#22C55E",
          soft: "rgba(34, 197, 94, 0.12)",
        },
      },
      fontFamily: {
        mono: '"JetBrains Mono", "Cascadia Code", Consolas, ui-monospace, monospace',
      },
    },
  },
  plugins: [],
};
