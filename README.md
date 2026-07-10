<div align="center">

# 📄 PaperCoat

**Turn any novel PDF into a convincing academic paper.**

*Stay productive. Read fiction.*

[![CI](https://github.com/malamutes/papercoat/actions/workflows/ci.yml/badge.svg)](https://github.com/malamutes/papercoat/actions/workflows/ci.yml)
[![Release](https://github.com/malamutes/papercoat/actions/workflows/release.yml/badge.svg)](https://github.com/malamutes/papercoat/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/papercoat)](https://crates.io/crates/papercoat)
[![License: MIT](https://img.shields.io/badge/license-MIT-purple)](LICENSE)

---

```bash
curl -fsSL https://raw.githubusercontent.com/malamutes/papercoat/main/install.sh | sh
papercoat ~/Downloads/novel.pdf
```

</div>

## Features

| | |
|---|---|
| ⚡ **Single binary** | 2.5 MB, zero dependencies, one command |
| 🖥️ **Interactive TUI** | File browser, live preview, keyboard-driven |
| 🎭 **Deep disguise** | Fake DOI, volume/issue, journal metadata, ORCIDs, dates |
| 📚 **3 journal templates** | Default (two-column), Nature, IEEE |
| 📊 **Stats mode** | Word count, readability scores, sentence stats |
| 📂 **Batch mode** | Process entire directories of PDFs |
| ⚙️ **Persistent config** | `~/.config/papercoat/config.json` |
| 🔗 **Auto-compile** | LaTeX → PDF if `pdflatex` available, with metadata injection |
| 🎨 **Beautiful terminal** | Purple/indigo theme, progress spinners, color-coded output |

## Quick Install

### macOS / Linux (one-liner)

```bash
curl -fsSL https://raw.githubusercontent.com/malamutes/papercoat/main/install.sh | sh
```

### Cargo

```bash
cargo install papercoat
```

### Homebrew (once tap is set up)

```bash
brew install papercoat/tap/papercoat
```

### Manual

Download the prebuilt binary for your platform from the [latest release](https://github.com/malamutes/papercoat/releases/latest), then:

```bash
chmod +x papercoat && sudo mv papercoat /usr/local/bin/
```

## Usage

```bash
# Basic — generate a LaTeX academic paper
papercoat novel.pdf

# Specify output, template, and format
papercoat novel.pdf -o paper.tex -t nature

# Generate PDF directly (requires pdflatex)
papercoat novel.pdf -f pdf

# Interactive TUI mode
papercoat

# Batch process all PDFs in a directory
papercoat ~/Downloads/

# Just show text statistics
papercoat novel.pdf --stats

# Override title and author
papercoat novel.pdf --title "A Computational Analysis..." --author "P. Reader"

# Open the result after generation
papercoat novel.pdf --open
```

### Templates

| Name | Description |
|---|---|
| `default` | Two-column, Times Roman (standard journal style) |
| `nature` | Single-column Nature-style layout |
| `ieee` | IEEE Transactions conference format |

### What comes out

```latex
\title{"A Long Way Gone": Computational Analysis of Contemporary Narrative Prose}
\author{L. Chen, K. Petrov, et al.}
\journal{Journal of Computational Literary Studies}
\doi{10.102/jcls.5354.1190}
\received{March 15, 2025}
\accepted{November 10, 2025}
```

A complete, compilable LaTeX document with:
- Structured abstract (Background / Methods / Results / Conclusions)
- 6 sections (Introduction through Conclusion)
- Inline Harvard-style citations
- Block quotes from the original text
- Author ORCIDs and affiliations
- 7 automatically generated references
- Acknowledgements, data availability, competing interests

## Screenshots

```
                          ┌─────────────────────────────┐
  ┌──────────────────────┐│  ⚙️  Options                 │
  │  📂 ~/Downloads/    ││  Template                    │
  │  📄 we_2.pdf       ││  ▸ default                   │
  │  📄 chapter3.pdf   ││    nature                    │
  │  📄 notes.pdf      ││    ieee                      │
  │                      ││                             │
  │  ↑↓ Navigate        ││  Format                      │
  │  Enter Select       ││  ▸ LaTeX (.tex)              │
  │  Backspace Up       ││    PDF (via LaTeX)            │
  └──────────────────────┘└─────────────────────────────┘
```

*Interactive TUI — navigate, preview, and generate with keyboard controls.*

## How it works

1. **Extract** — Text is extracted from the PDF using `pdf-extract` (with optional OCR via `pytesseract`)
2. **Transform** — The raw text is analyzed and structured into a paper: title, authors, abstract, sections with block quotes, and references
3. **Render** — Output is generated as LaTeX (or compiled to PDF if `pdflatex` is available)
4. **Disguise** — Fake DOI, journal metadata, ORCIDs, and review dates are injected for realism

## Development

```bash
git clone https://github.com/malamutes/papercoat.git
cd papercoat
cargo build
cargo test
```

### Release

```bash
git tag v1.0.0 && git push --tags
# GitHub Actions builds binaries for all platforms
```

## Why?

Some of the best books are read during lunch breaks, commutes, and those "I'm reviewing a document" moments. PaperCoat makes it look like you're doing serious research.

## License

MIT
