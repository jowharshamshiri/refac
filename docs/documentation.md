---
title: Documentation
layout: default
permalink: /documentation/
---

## Core Concepts

### 1. Operation Modes

Refac supports several operation modes:

- **Full mode**: Process both names and content (default)
- **Names-only**: Rename files/directories only
- **Content-only**: Modify file contents only
- **Files-only/Dirs-only**: Process specific item types

### 2. Safety Features

- **Collision Detection**: Prevents overwriting existing items
- **Binary Protection**: Automatically skips binary files
- **Dry-run Mode**: Preview changes before applying
- **Backup System**: Optional pre-modification backups

### 3. Matching

```bash
# Case-insensitive matching
refac . -i "oldname" "newname"

# Regex patterns
refac . -r "\b\d{4}-\d{2}-\d{2}\b" "DATE"
```

[View Full API Reference](/usage){: .btn .btn-outline }
