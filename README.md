# memory-plimpsest

> **Layered memory with ghost traces — write without erasing**

[![crates.io](https://img.shields.io/crates/v/memory-plimpsest.svg)](https://crates.io/crates/memory-plimpsest)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A palimpsest is an ancient manuscript where older writing shows through as ghost traces beneath newer text. This crate implements the same concept for agent memory:

- **Layers**: Each write creates a new layer on top
- **Ghost traces**: Older layers are still visible (faded)
- **Read-through**: Read the top layer, or drill down for history
- **Decay**: Older layers fade over time

The palimpsest metaphor solves a key problem in agent memory: how to update beliefs without losing the history of what was believed before.

## Installation

```toml
[dependencies]
memory-plimpsest = "0.1.0"
```

## License

MIT © [SuperInstance](https://github.com/SuperInstance)

---

*Part of the [Exocortex](https://github.com/SuperInstance/exocortex) project — persistent cognitive substrate for multi-agent systems.*
