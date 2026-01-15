# Linguisto

[English](./README.md)

[![NPM version](https://img.shields.io/npm/v/@homy/linguist.svg?style=flat)](https://npmjs.com/package/@homy/linguist) [![NPM downloads](https://img.shields.io/npm/dm/@homy/linguist.svg?style=flat)](https://npmjs.com/package/@homy/linguist) [![License](https://img.shields.io/npm/l/@homy/linguist.svg?style=flat)](./LICENSE)

## 简介

**Linguisto** 是一个基于 [github-linguist](https://github.com/github-linguist/linguist) 的高性能代码语言分析工具。它基于 Rust 开发，并通过 [NAPI-RS](https://napi.rs/) 提供 Node.js 绑定，能够快速扫描目录，统计文件数量、计算字节大小并确定语言占比，同时智能地过滤第三方依赖和被忽略的文件。

## 功能特性

- **卓越性能**：底层使用 Rust 编写，利用多线程能力快速遍历文件系统。
- **智能过滤**：自动尊重 `.gitignore` 设置，跳过隐藏文件，并能识别并排除第三方库（如 `node_modules`）。
- **精准识别**：基于成熟的语言检测算法，支持通过文件名、扩展名及内容歧义消除来确定语言。
- **精美输出**：提供色彩丰富的终端 UI 进度条展示，支持按字节数或文件数进行排序。
- **数据集成**：支持导出 JSON 格式，方便与其他工具集成。
- **跨平台支持**：支持 macOS, Linux, Windows 以及 WASI 环境。

## 目录

<!-- toc -->

- [安装](#安装)
- [使用方法](#使用方法)
  - [命令行 (CLI) 使用](#命令行-cli-使用)
  - [编程方式使用](#编程方式使用)
- [API 参考](#api-参考)
  - [analyzeDirectory(dir)](#analyzedirectorydir)
  - [analyzeDirectorySync(dir)](#analyzedirectorysyncdir)
  - [LanguageStat](#languagestat)
- [许可证](#许可证)

<!-- tocstop -->

## 安装

### 命令行工具

如果你已安装 Rust 环境，可以直接通过 Cargo 安装：

```bash
cargo install linguisto
```

或者通过 npm 全局安装：

```bash
npm install -g @homy/linguist
```

### 编程接口

在你的 Node.js 项目中作为依赖安装：

```bash
npm install @homy/linguist
```

## 使用方法

### 命令行 (CLI) 使用

在当前目录下运行，即可看到直观的语言占比图表（默认按照文件大小排序）：

```bash
linguisto
```

分析指定目录：

```bash
linguisto /path/to/your/project
```

#### 常用选项

- `--json`: 以 JSON 格式输出结果。
- `--all`: 显示所有检测到的文件（默认仅显示编程语言，过滤掉部分配置文件等）。
- `--sort <type>`: 排序方式。`type` 可以是 `file_count`（按文件数降序）或 `bytes`（按字节数降序，默认）。

#### 示例

```bash
# 获取当前项目的 JSON 统计结果并按文件数排序
linguisto . --json --sort=file_count
```

### 编程方式使用

你可以直接在 Node.js 或 TypeScript 代码中调用 `@homy/linguist` 提供的 API。

```javascript
const { analyzeDirectory, analyzeDirectorySync } = require('@homy/linguist');

// 异步分析（推荐用于大型目录）
async function run() {
  const stats = await analyzeDirectory('./src');
  console.log(stats);
}

run();

// 同步分析
const syncStats = analyzeDirectorySync('./src');
console.log(syncStats);
```

## API 参考

### analyzeDirectory(dir)

- 类型: `(dir: string) => Promise<LanguageStat[]>`

异步分析目标目录并返回包含语言统计信息的数组。

### analyzeDirectorySync(dir)

- 类型: `(dir: string) => LanguageStat[]`

同步分析目标目录并返回包含语言统计信息的数组。

### LanguageStat

每个统计对象包含以下字段：

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| `lang` | `string` | 检测到的语言名称（如 "Rust", "TypeScript"） |
| `count` | `number` | 该语言的文件数量 |
| `bytes` | `number` | 该语言文件占用的总字节数 |
| `ratio` | `number` | 在整个项目中的占比 (0.0 - 1.0) |
| `isProgramming` | `boolean` | 是否为编程语言 |

## 鸣谢 (Credits)

- [github-linguist/linguist](https://github.com/github-linguist/linguist) - 该项目为本工具提供了灵感及语言检测逻辑。
- [drshade/linguist](https://github.com/drshade/linguist) - 作为参考的 Rust 语言检测实现。

## 许可证

[MIT](./LICENSE) © [Homyee King](https://github.com/HomyeeKing)
