# Linguisto

[简体中文](./README-zh.md)

[![NPM version](https://img.shields.io/npm/v/@homy/linguist.svg?style=flat)](https://npmjs.com/package/@homy/linguist) [![NPM downloads](https://img.shields.io/npm/dm/@homy/linguist.svg?style=flat)](https://npmjs.com/package/@homy/linguist) [![License](https://img.shields.io/npm/l/@homy/linguist.svg?style=flat)](./LICENSE)

## Introduction

**Linguisto** is a high-performance code language analysis tool based on [github-linguist](https://github.com/github-linguist/linguist). Built with Rust and providing Node.js bindings via [NAPI-RS](https://napi.rs/), it quickly scans directories to count files, calculate byte sizes, and determine language percentages, while intelligently filtering out third-party dependencies and ignored files.


![show-case](https://github.com/user-attachments/assets/e8f41431-1e55-40fb-9940-087671c0867c)




## Features

- **Superior Performance**: Written in Rust, leveraging multi-threading for fast file system traversal.
- **Smart Filtering**: Automatically respects `.gitignore`, skips hidden files, and excludes vendored files (e.g., `node_modules`).
- **Precise Detection**: Based on robust language detection algorithms, supporting filename, extension, and content-based disambiguation.
- **Beautiful Output**: Provides a colorful terminal UI with progress bars, supporting sorting by bytes or file count.
- **Data Integration**: Supports JSON output for easy integration with other tools.
- **Cross-platform**: Supports macOS, Linux, Windows, and WASI environments.

## Table of Contents

<!-- toc -->

- [Install](#install)
- [Usage](#usage)
  - [CLI Usage](#cli-usage)
  - [Programmatic Usage](#programmatic-usage)
- [Benchmark](#benchmark)
- [References](#references)
  - [analyzeDirectory(dir)](#analyzedirectorydir)
  - [analyzeDirectorySync(dir)](#analyzedirectorysyncdir)
  - [LanguageStat](#languagestat)
- [License](#license)

<!-- tocstop -->

## Install

### For CLI

If you have Rust installed, you can install it via Cargo:

```bash
cargo install linguisto
```

Or install it globally via npm:

```bash
npm install -g @homy/linguist
```

### For API

Install it as a dependency in your Node.js project:

```bash
npm install @homy/linguist
```

## Usage

### CLI Usage

Run it in the current directory to see an intuitive language distribution chart (sorted by byte size by default):

```bash
linguisto
```

Analyze a specific directory:

```bash
linguisto /path/to/your/project
```

#### Common Options

- `--json`: Output results in JSON format.
- `--all`: Show all calculated language statistics. (Currently behaves the same as the default view, kept for compatibility.)
- `--sort <type>`: Sort results. `type` can be `file_count` (descending) or `bytes` (descending, default).
- `--max-lang <number>`: Maximum number of languages to display individually. Remaining languages will be grouped into "Other" (default: 6).

#### Example

```bash
# Get JSON stats for the current project sorted by file count
linguisto . --json --sort=file_count
```

### Programmatic Usage

You can call the API provided by `@homy/linguist` directly in your Node.js or TypeScript code.

```javascript
const { analyzeDirectory, analyzeDirectorySync } = require('@homy/linguist');

// Asynchronous analysis (recommended for large directories)
async function run() {
  const stats = await analyzeDirectory('./src');
  console.log(stats);
}

run();

// Synchronous analysis
const syncStats = analyzeDirectorySync('./src');
console.log(syncStats);
```

## Benchmark

This project includes a simple benchmark comparing **linguisto (native Rust + NAPI)** with the pure JavaScript implementation **linguist-js**.

The benchmark script is located at [`benchmark/bench.ts`](./benchmark/bench.ts) and can be run with:

```bash
pnpm bench
```

Below is a sample result on this repository (macOS, Node.js, default settings):

```text
Running benchmark...
┌─────────┬──────────────────────┬────────────────────┬─────────────────────┬────────────────────────┬────────────────────────┬─────────┐
│ (index) │ Task name            │ Latency avg (ns)   │ Latency med (ns)    │ Throughput avg (ops/s) │ Throughput med (ops/s) │ Samples │
├─────────┼──────────────────────┼────────────────────┼─────────────────────┼────────────────────────┼────────────────────────┼─────────┤
│ 0       │ 'linguist-js'        │ '87352945 ± 0.86%' │ '86415417 ± 955312' │ '11 ± 0.79%'           │ '12 ± 0'               │ 64      │
│ 1       │ 'linguisto (native)' │ '2903259 ± 2.20%'  │ '3118667 ± 52375'   │ '363 ± 2.64%'          │ '321 ± 5'              │ 345     │
└─────────┴──────────────────────┴────────────────────┴─────────────────────┴────────────────────────┴────────────────────────┴─────────┘
```

From this run, **linguisto (native)** achieves roughly **30× higher throughput** than **linguist-js** on this project, thanks to Rust's multi-threaded file system traversal and native execution.

Let's a real world case in `vscode` codebase :
| linguisto | linguist-js |
| :--- | :--- |
| ![show-case](https://github.com/user-attachments/assets/e8f41431-1e55-40fb-9940-087671c0867c) | ![linguist-js](https://github.com/user-attachments/assets/3bfda736-f40d-4801-81a8-b1377e5192f4) |

## References

### analyzeDirectory(dir)

- Type: `(dir: string) => Promise<LanguageStat[]>`

Asynchronously analyzes the target directory and returns an array of language statistics.

### analyzeDirectorySync(dir)

- Type: `(dir: string) => LanguageStat[]`

Synchronously analyzes the target directory and returns an array of language statistics.

### LanguageStat

Each statistical object contains the following fields:

| Field | Type | Description |
| :--- | :--- | :--- |
| `lang` | `string` | Detected language name (e.g., "Rust", "TypeScript") |
| `count` | `number` | Number of files for this language |
| `bytes` | `number` | Total bytes occupied by files of this language |
| `ratio` | `number` | Percentage in the overall project (0.0 - 1.0) |

## Credits

- [github-linguist/linguist](https://github.com/github-linguist/linguist) - The project that inspired this tool and provides the language detection logic.
- [drshade/linguist](https://github.com/drshade/linguist) - A Rust implementation of linguist that served as a reference.

## License

[MIT](./LICENSE) © [Homyee King](https://github.com/HomyeeKing)
