---
title: Contributing
layout: default
---

## Development Setup

1. Fork the repository
2. Clone your fork:

```bash
git clone https://github.com/your-username/nomion
```

3. Build the project:

```bash
cargo build
```

## Testing

Run the full test suite:

```bash
cargo test --all-features
```

## Contribution Guidelines

- Follow Rust coding conventions
- Include tests for new features
- Update documentation accordingly
- Use descriptive commit messages
- Open an issue before major changes

## Code Structure

```
src/
├── cli.rs        # Command-line interface
├── engine.rs     # Core processing logic
├── fs_ops.rs     # File system operations
└── detector.rs   # Collision detection
```
