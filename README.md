# word-blazer

![CI](https://github.com/mmed-hajnasr/word-blazer/actions/workflows/ci.yml/badge.svg)
[![Release](https://img.shields.io/github/v/release/mmed-hajnasr/word-blazer?style=flat-square)](https://github.com/mmed-hajnasr/word-blazer/releases)
[![Last Commit](https://img.shields.io/github/last-commit/mmed-hajnasr/word-blazer/main?style=flat-square)](https://github.com/mmed-hajnasr/word-blazer/commits/main)

## Game overview

a TUI Labyrinth game that takes advatage of multiple graph alguorithms.

> [!TIP]
>
> The game follows the font and colors of your terminal emulator.

| ![dark demo](./resources/dark-demo.png) | ![light demo](./resources/light-demo.png) |
| --------------------------------------- | ----------------------------------------- |

## How to play

Navigate through the labyrinth to find the hidden exit (★) while gathering letters to form words. Watch your step count, each move costs one precious step. Don't worry though! You can form words to earn more steps and boost your score. The longer your words, the higher your score climbs. Plan your route wisely, explore every corner, and turn those scattered letters into lexical gold as you hunt for the elusive exit.  
Think you're ready to become a word-blazer master?

## Installation

### Pre-compiled binary.

1. **Download the Binary:**

Head over to the project's releases on GitHub: [v0.1.0 release](https://github.com/mmed-hajnasr/word-blazer/releases/tag/v0.1.0). There, you'll find pre-built binaries for various operating systems. Download the binary that corresponds to your system.

2. **Extract and run the binary:**

```sh
 tar -xvzf [downloaded_file]
 ./word-blazer
```

### Manual installation

```sh
git clone git@github.com:mmed-hajnasr/word-blazer.git
cd word-blazer
cargo install --path .
```

## Possible future improvements

- [ ] more PowerUps
- [ ] main menu
- [ ] different modes
- [ ] refactor labyrinth.rs
