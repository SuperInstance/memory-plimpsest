# memory-plimpsest

> **Layered memory with ghost traces — write, fade, and read through layers like an ancient palimpsest**

[![crates.io](https://img.shields.io/crates/v/memory-plimpsest.svg)](https://crates.io/crates/memory-plimpsest)
[![docs.rs](https://docs.rs/memory-plimpsest/badge.svg)](https://docs.rs/memory-plimpsest)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What is a Palimpsest?

A **palimpsest** is an ancient manuscript where the original writing was scraped off and new text was written over it — but traces of the older text remain visible beneath the surface. Archaeologists and historians can read through the layers to recover lost knowledge.

This crate brings that metaphor to agent memory. Instead of a simple key-value store where writes overwrite previous values, memory-plimpsest maintains a **stack of semi-transparent layers**. Each new write adds a layer on top. Older layers gradually fade (decay in opacity) but remain readable — as "ghost traces" — until they eventually decay completely.

This model is particularly suited for AI agents and cognitive systems where:
- **Recent context** is most important (top layers, full opacity)
- **Past context** shouldn't vanish instantly (ghost layers, faded but visible)
- **Very old context** naturally fades away (fully decayed layers, pruned)

## Why Does This Matter?

Traditional memory systems face a tension between persistence and relevance. Keep everything and you drown in stale data. Overwrite aggressively and you lose important context. The palimpsest model resolves this:

- **Gradual decay**: Information doesn't disappear in a binary on/off — it fades, giving downstream systems a chance to rescue it
- **Read-through**: Access the most recent state instantly, or drill down to historical layers
- **Ghost traces**: Even faded layers leave imprints — useful for pattern detection and context recovery
- **Automatic pruning**: Fully-decayed layers are removed without manual cleanup

Real-world applications:
- **Agent memory**: Maintain conversation context that fades naturally as topics shift
- **Sensor data**: Recent readings are sharp, older readings fade but remain comparable
- **Edit history**: Track document revisions with natural emphasis on recent changes
- **Cognitive modeling**: Simulate human-like memory decay curves in AI systems

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                  Memory Palimpsest Model                      │
│                                                              │
│  Write Stack (newest on top, oldest on bottom)               │
│                                                              │
│  Layer 3 [t=300] ████████████ opacity: 1.00  ← newest       │
│  Layer 2 [t=200] ████████░░░░ opacity: 0.72                 │
│  Layer 1 [t=100] ████░░░░░░░░ opacity: 0.36    ← fading     │
│  Layer 0 [t=50]  ██░░░░░░░░░░ opacity: 0.12    ← ghost      │
│                                                              │
│          │                                                   │
│          ▼ Decay: opacity *= decay_factor each write         │
│          ▼ Prune: remove layers with opacity → 0             │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  ReadThrough │  │  GhostTrace  │  │  LayerDecay  │       │
│  │  read_top()  │  │  Faded but   │  │  Configurable│       │
│  │  drill_down()│  │  still there │  │  decay rate  │       │
│  │  search()    │  │  ~content~   │  │  + min opacity│      │
│  │  composite() │  │              │  │              │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└──────────────────────────────────────────────────────────────┘
```

## Quick Start

```rust
use memory_plimpsest::{MemoryPlimpsest, LayerDecay, ReadThrough};

// Create a palimpsest with custom decay (opacity ×0.7 per new write)
let decay = LayerDecay::new(0.7, 0.01);
let mut palimpsest = MemoryPlimpsest::new(decay);

// Write layers — each new write decays all existing layers
palimpsest.write("Agent started monitoring emails", 1000);
palimpsest.write("Found 3 urgent messages", 2000);
palimpsest.write("Responded to urgent thread", 3000);

// Read the top (most recent) layer
let rt = ReadThrough::new(&palimpsest);
assert_eq!(rt.read_top(), Some("Responded to urgent thread"));

// Drill down through visible layers (opacity > 0.5)
let visible = rt.drill_down(0.5);
println!("Visible layers: {:?}", visible);
```

### Searching and Composites

```rust
// Search for specific content (searches from newest to oldest)
let result = rt.search("urgent");
if let Some((layer_idx, content)) = result {
    println!("Found '{}' at layer {}", content, layer_idx);
}

// Get a composite view of all layers
let composite = rt.composite();
println!("Full context: {}", composite);
// Output: "Agent started... / [ghost: Found 3...] / Responded to..."
```

### Working with Ghost Traces

```rust
// After several writes, older layers become ghosts (opacity < 0.5)
palimpsest.write("New task assigned", 4000);
palimpsest.write("Task completed", 5000);

let ghosts = palimpsest.ghost_traces();
for ghost in &ghosts {
    println!("Ghost from layer {}: {}", ghost.layer_index, ghost.render());
    // "~Agent started monitoring emails~" or "[too faded to read]"
}
```

### Pruning Decayed Layers

```rust
// Remove fully-decayed layers to reclaim memory
let pruned_count = palimpsest.prune();
println!("Pruned {} fully-decayed layers", pruned_count);
println!("Active layers: {}", palimpsest.layer_count());
```

## API Reference

### PlimpsestLayer

| Field | Type | Description |
|-------|------|-------------|
| `content` | `String` | Text content of this layer |
| `timestamp` | `u64` | Unix timestamp when written |
| `opacity` | `f64` | Current opacity (0.0–1.0, decays over time) |

| Method | Returns | Description |
|--------|---------|-------------|
| `PlimpsestLayer::new(content, timestamp)` | `PlimpsestLayer` | Create layer at full opacity |
| `PlimpsestLayer::with_opacity(content, timestamp, opacity)` | `PlimpsestLayer` | Create layer at given opacity |
| `layer.is_ghost()` | `bool` | True if opacity < 0.1 |

### MemoryPlimpsest

| Method | Returns | Description |
|--------|---------|-------------|
| `MemoryPlimpsest::new(decay)` | `MemoryPlimpsest` | Create with decay configuration |
| `palimpsest.write(content, timestamp)` | `()` | Write a new layer (decays all existing) |
| `palimpsest.top()` | `Option<&PlimpsestLayer>` | Newest layer |
| `palimpsest.layer(index)` | `Option<&PlimpsestLayer>` | Access layer by index |
| `palimpsest.layer_count()` | `usize` | Number of layers |
| `palimpsest.ghost_traces()` | `Vec<GhostTrace>` | Layers with opacity < 0.5 |
| `palimpsest.prune()` | `usize` | Remove fully-decayed layers |

### ReadThrough

| Method | Returns | Description |
|--------|---------|-------------|
| `ReadThrough::new(&palimpsest)` | `ReadThrough` | Create accessor |
| `rt.read_top()` | `Option<&str>` | Top layer content |
| `rt.drill_down(min_opacity)` | `Vec<&str>` | All layers above threshold |
| `rt.search(query)` | `Option<(usize, &str)>` | Find content (newest-first) |
| `rt.composite()` | `String` | Merge all layers into one string |

### LayerDecay

| Field | Default | Description |
|-------|---------|-------------|
| `decay_factor` | 0.85 | Multiplier applied per write (0.0–1.0) |
| `min_opacity` | 0.01 | Below this, layer is set to 0.0 |

### GhostTrace

| Method | Returns | Description |
|--------|---------|-------------|
| `GhostTrace::new(content, fadedness, layer_index)` | `GhostTrace` | Create a ghost trace |
| `ghost.render()` | `String` | `"~content~"` or `"[too faded to read]"` |

## Mathematical Background

### Exponential Decay Model

Each layer's opacity follows a geometric decay:
```
opacityₖ(t) = opacity₀ × decay_factorⁿ
```
where n is the number of new layers written since layer k was created, and `decay_factor ∈ [0, 1]`.

### Half-Life

The effective half-life of a memory layer (number of writes until opacity drops to 0.5):
```
half_life = -ln(2) / ln(decay_factor)
```

| decay_factor | Half-life (writes) | Time to ghost (< 0.1) |
|:---:|:---:|:---:|
| 0.95 | 13.5 | 44.9 |
| 0.85 | 4.3 | 14.2 |
| 0.70 | 1.9 | 6.4 |
| 0.50 | 1.0 | 3.3 |

### Forgetting Curve Connection

This model mirrors **Ebbinghaus's forgetting curve** (1885), which describes how memory retention decays exponentially over time:
```
R(t) = e^(-t/S)
```
where R is retention and S is the "strength" of the memory. The palimpsest's decay_factor approximates this: each new write represents a time step, and opacity models retention.

## Installation

```bash
cargo add memory-plimpsest
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
memory-plimpsest = "0.1.0"
```

## Related Crates

- [`constellation-map`](https://github.com/SuperInstance/constellation-map) — Fleet visualization as star charts
- [`knowledge-compass`](https://github.com/SuperInstance/knowledge-compass) — Provenance navigation for knowledge graphs
- [`emotional-colorist`](https://github.com/SuperInstance/emotional-colorist) — Valence-based color mapping
- [`cortex-toml`](https://github.com/SuperInstance/cortex-toml) — Configuration-as-code for Exocortex

## License

MIT © [SuperInstance](https://github.com/SuperInstance)

---

*Part of the [Exocortex](https://github.com/SuperInstance/exocortex) project — persistent cognitive substrate for multi-agent systems.*
