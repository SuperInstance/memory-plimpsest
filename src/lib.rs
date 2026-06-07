//! # memory-plimpsest
//!
//! Layered memory system with ghost traces. Writes stack as semi-transparent layers;
//! older content fades but remains readable through the palimpsest model.
//!
//! ## Core Types
//! - [`PlimpsestLayer`] — A single write layer with timestamp and opacity
//! - [`GhostTrace`] — A faded previous content still visible beneath newer layers
//! - [`MemoryPlimpsest`] — The main stack of layers, newest on top, oldest ghosted
//! - [`ReadThrough`] — Read the top layer, or drill down for older content
//! - [`LayerDecay`] — Older layers fade over time: opacity *= decay_factor

/// A single write layer with content, timestamp, and opacity.
#[derive(Debug, Clone)]
pub struct PlimpsestLayer {
    /// Text content of this layer.
    pub content: String,
    /// Unix timestamp (seconds) when this layer was written.
    pub timestamp: u64,
    /// Opacity from 0.0 (invisible) to 1.0 (fully opaque).
    pub opacity: f64,
}

impl PlimpsestLayer {
    /// Create a new layer with the given content, timestamp, and full opacity.
    pub fn new(content: impl Into<String>, timestamp: u64) -> Self {
        Self {
            content: content.into(),
            timestamp,
            opacity: 1.0,
        }
    }

    /// Create a layer with explicit opacity.
    pub fn with_opacity(content: impl Into<String>, timestamp: u64, opacity: f64) -> Self {
        Self {
            content: content.into(),
            timestamp,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }

    /// Returns true if this layer is effectively invisible.
    pub fn is_ghost(&self) -> bool {
        self.opacity < 0.1
    }
}

/// A faded trace of previous content still visible beneath newer layers.
#[derive(Debug, Clone)]
pub struct GhostTrace {
    /// The original content.
    pub content: String,
    /// How faded this trace is (0.0 = gone, 1.0 = original).
    pub fadedness: f64,
    /// Which layer index this ghost came from.
    pub layer_index: usize,
}

impl GhostTrace {
    /// Create a new ghost trace.
    pub fn new(content: impl Into<String>, fadedness: f64, layer_index: usize) -> Self {
        Self {
            content: content.into(),
            fadedness: fadedness.clamp(0.0, 1.0),
            layer_index,
        }
    }

    /// Render the ghost content with fade indicators.
    pub fn render(&self) -> String {
        if self.fadedness < 0.1 {
            String::from("[too faded to read]")
        } else {
            format!("~{}~", self.content)
        }
    }
}

/// Configuration for layer decay behavior.
#[derive(Debug, Clone)]
pub struct LayerDecay {
    /// Factor by which opacity is multiplied per decay step (0.0–1.0).
    pub decay_factor: f64,
    /// Minimum opacity before a layer is considered fully decayed.
    pub min_opacity: f64,
}

impl Default for LayerDecay {
    fn default() -> Self {
        Self {
            decay_factor: 0.85,
            min_opacity: 0.01,
        }
    }
}

impl LayerDecay {
    /// Create a new decay configuration.
    pub fn new(decay_factor: f64, min_opacity: f64) -> Self {
        Self {
            decay_factor: decay_factor.clamp(0.0, 1.0),
            min_opacity: min_opacity.clamp(0.0, 1.0),
        }
    }

    /// Apply one decay step to a layer's opacity.
    pub fn apply(&self, layer: &mut PlimpsestLayer) {
        layer.opacity *= self.decay_factor;
        if layer.opacity < self.min_opacity {
            layer.opacity = 0.0;
        }
    }
}

/// A stack of palimpsest layers — newest on top, oldest ghosted.
#[derive(Debug, Clone)]
pub struct MemoryPlimpsest {
    layers: Vec<PlimpsestLayer>,
    decay: LayerDecay,
}

impl Default for MemoryPlimpsest {
    fn default() -> Self {
        Self::new(LayerDecay::default())
    }
}

impl MemoryPlimpsest {
    /// Create a new empty palimpsest with the given decay config.
    pub fn new(decay: LayerDecay) -> Self {
        Self {
            layers: Vec::new(),
            decay,
        }
    }

    /// Write a new layer on top.
    pub fn write(&mut self, content: impl Into<String>, timestamp: u64) {
        // Apply decay to all existing layers
        for layer in &mut self.layers {
            self.decay.apply(layer);
        }
        self.layers.push(PlimpsestLayer::new(content, timestamp));
    }

    /// Number of layers in the palimpsest.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get a reference to the top (newest) layer.
    pub fn top(&self) -> Option<&PlimpsestLayer> {
        self.layers.last()
    }

    /// Get a reference to a layer by index (0 = oldest).
    pub fn layer(&self, index: usize) -> Option<&PlimpsestLayer> {
        self.layers.get(index)
    }

    /// Collect all ghost traces (layers with opacity < 0.5).
    pub fn ghost_traces(&self) -> Vec<GhostTrace> {
        self.layers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.opacity < 0.5)
            .map(|(i, l)| GhostTrace::new(&l.content, l.opacity, i))
            .collect()
    }

    /// Remove all fully-decayed layers.
    pub fn prune(&mut self) -> usize {
        let before = self.layers.len();
        self.layers.retain(|l| l.opacity > 0.0);
        before - self.layers.len()
    }
}

/// Read-through accessor for palimpsest layers.
#[derive(Debug)]
pub struct ReadThrough<'a> {
    palimpsest: &'a MemoryPlimpsest,
}

impl<'a> ReadThrough<'a> {
    /// Create a read-through accessor.
    pub fn new(palimpsest: &'a MemoryPlimpsest) -> Self {
        Self { palimpsest }
    }

    /// Read the top layer's content, or None if empty.
    pub fn read_top(&self) -> Option<&str> {
        self.palimpsest.top().map(|l| l.content.as_str())
    }

    /// Drill down through layers, returning content from oldest to newest
    /// that have opacity above the given threshold.
    pub fn drill_down(&self, min_opacity: f64) -> Vec<&str> {
        self.palimpsest
            .layers
            .iter()
            .filter(|l| l.opacity >= min_opacity)
            .map(|l| l.content.as_str())
            .collect()
    }

    /// Find the first layer (from top) whose content contains the query.
    pub fn search(&self, query: &str) -> Option<(usize, &str)> {
        self.palimpsest
            .layers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, l)| l.content.contains(query))
            .map(|(i, l)| (i, l.content.as_str()))
    }

    /// Merge all visible layers into a single composite string.
    pub fn composite(&self) -> String {
        self.palimpsest
            .layers
            .iter()
            .map(|l| {
                if l.opacity > 0.5 {
                    l.content.clone()
                } else {
                    format!("[ghost: {}]", l.content)
                }
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = PlimpsestLayer::new("hello", 1000);
        assert_eq!(layer.content, "hello");
        assert_eq!(layer.timestamp, 1000);
        assert!((layer.opacity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_layer_with_opacity() {
        let layer = PlimpsestLayer::with_opacity("faded", 500, 0.3);
        assert!((layer.opacity - 0.3).abs() < f64::EPSILON);
        assert!(!layer.is_ghost());
    }

    #[test]
    fn test_layer_is_ghost() {
        let layer = PlimpsestLayer::with_opacity("x", 0, 0.05);
        assert!(layer.is_ghost());
    }

    #[test]
    fn test_ghost_trace_render() {
        let visible = GhostTrace::new("content", 0.5, 0);
        assert_eq!(visible.render(), "~content~");

        let faded = GhostTrace::new("old", 0.05, 1);
        assert_eq!(faded.render(), "[too faded to read]");
    }

    #[test]
    fn test_decay_apply() {
        let decay = LayerDecay::new(0.5, 0.01);
        let mut layer = PlimpsestLayer::new("test", 0);
        decay.apply(&mut layer);
        assert!((layer.opacity - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_decay_to_zero() {
        let decay = LayerDecay::new(0.01, 0.1);
        let mut layer = PlimpsestLayer::new("test", 0);
        decay.apply(&mut layer);
        assert!((layer.opacity).abs() < 1e-9);
    }

    #[test]
    fn test_write_layers() {
        let mut p = MemoryPlimpsest::default();
        p.write("first", 100);
        p.write("second", 200);
        assert_eq!(p.layer_count(), 2);
        assert_eq!(p.top().unwrap().content, "second");
    }

    #[test]
    fn test_write_applies_decay() {
        let decay = LayerDecay::new(0.5, 0.0);
        let mut p = MemoryPlimpsest::new(decay);
        p.write("first", 100);
        assert!((p.layer(0).unwrap().opacity - 1.0).abs() < 1e-9);
        p.write("second", 200);
        // First layer should have decayed
        assert!((p.layer(0).unwrap().opacity - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_ghost_traces() {
        let decay = LayerDecay::new(0.3, 0.0);
        let mut p = MemoryPlimpsest::new(decay);
        p.write("old", 100);
        p.write("mid", 200);
        p.write("new", 300);
        let ghosts = p.ghost_traces();
        assert!(ghosts.len() >= 1);
    }

    #[test]
    fn test_prune() {
        let decay = LayerDecay::new(0.01, 0.1);
        let mut p = MemoryPlimpsest::new(decay);
        p.write("will_decay", 100);
        p.write("triggers_decay", 200);
        // After second write, first layer decayed to 0 (0.01 < 0.1 min)
        let pruned = p.prune();
        assert!(pruned >= 1);
    }

    #[test]
    fn test_read_through_top() {
        let mut p = MemoryPlimpsest::default();
        let rt = ReadThrough::new(&p);
        assert!(rt.read_top().is_none());

        p.write("hello", 100);
        let rt = ReadThrough::new(&p);
        assert_eq!(rt.read_top().unwrap(), "hello");
    }

    #[test]
    fn test_read_through_drill_down() {
        let mut p = MemoryPlimpsest::default();
        p.write("layer1", 100);
        p.write("layer2", 200);
        let rt = ReadThrough::new(&p);
        let visible = rt.drill_down(0.5);
        assert!(visible.len() >= 1);
    }

    #[test]
    fn test_read_through_search() {
        let mut p = MemoryPlimpsest::default();
        p.write("alpha", 100);
        p.write("beta", 200);
        p.write("gamma", 300);
        let rt = ReadThrough::new(&p);
        let result = rt.search("beta");
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, "beta");
    }

    #[test]
    fn test_composite() {
        let mut p = MemoryPlimpsest::default();
        p.write("hello", 100);
        let rt = ReadThrough::new(&p);
        assert!(rt.composite().contains("hello"));
    }
}
