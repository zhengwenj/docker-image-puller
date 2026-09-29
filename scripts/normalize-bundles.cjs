// 归一化打包产物文件名为 kebab-case:docker-image-puller-<version>-x64-<kind>.<ext>
// 说明:MSI/NSIS 文件名由 Tauri 按 productName(显示名)生成,这里在打包后统一重命名,
//       安装后开始菜单/控制面板里的显示名不受影响。
// 用法:node scripts/normalize-bundles.cjs(通常由 `pnpm run dist` 自动调用)
const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..");
const tauriConf = JSON.parse(
  fs.readFileSync(path.join(root, "src-tauri/tauri.conf.json"), "utf8"),
);
const productName = tauriConf.productName;
const version = tauriConf.version;

// 主程序 exe 名来自 Cargo 包名
const cargo = fs.readFileSync(path.join(root, "src-tauri/Cargo.toml"), "utf8");
const pkgName = cargo.match(/^name\s*=\s*"(.+)"/m)[1];

const bundleDir = path.join(root, "src-tauri/target/release/bundle");

function sizeMb(filePath) {
  return (fs.statSync(filePath).size / 1024 / 1024).toFixed(2);
}

// 安装包:重命名为 kebab-case
const installers = [
  ["msi", `${productName}_${version}_x64_en-US.msi`, `${pkgName}-${version}-x64-setup.msi`],
  ["nsis", `${productName}_${version}_x64-setup.exe`, `${pkgName}-${version}-x64-setup.exe`],
];
for (const [dir, source, target] of installers) {
  const sourcePath = path.join(bundleDir, dir, source);
  if (!fs.existsSync(sourcePath)) continue; // 对应 bundler 未启用时跳过
  const targetPath = path.join(bundleDir, dir, target);
  fs.rmSync(targetPath, { force: true });
  fs.renameSync(sourcePath, targetPath);
  console.log(`✓ ${dir}/${target} (${sizeMb(targetPath)} MiB)`);
}

// 免安装便携版:Windows 下主程序 exe 自包含,直接复制为单文件绿色版
const releaseDir = path.join(root, "src-tauri/target/release");
const exePath = path.join(releaseDir, `${pkgName}.exe`);
if (!fs.existsSync(exePath)) {
  console.error(`未找到主程序:${exePath},请先执行 tauri build`);
  process.exit(1);
}
const portableDir = path.join(bundleDir, "portable");
fs.mkdirSync(portableDir, { recursive: true });
for (const file of fs.readdirSync(portableDir)) {
  if (file.endsWith("-portable.zip") || file.endsWith("-portable.exe")) {
    fs.rmSync(path.join(portableDir, file), { force: true });
  }
}
const portableDest = path.join(portableDir, `${pkgName}-${version}-x64-portable.exe`);
fs.copyFileSync(exePath, portableDest);
console.log(`✓ portable/${pkgName}-${version}-x64-portable.exe (${sizeMb(portableDest)} MiB)`);
