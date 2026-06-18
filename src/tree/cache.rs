//! A cache for storing the results of layout computation

use crate::geometry::Size;
use crate::style::AvailableSpace;
use crate::tree::{LayoutInput, LayoutOutput, NodeId, RunMode};
use core::fmt;

/// The number of cache entries for each node in the tree
const CACHE_SIZE: usize = 9;

/// Opaque identity for one cache entry within a single node's layout cache.
///
/// This identifies the entry Taffy stored or selected. It is intentionally not
/// a public slot number and is meaningful only together with the [`NodeId`] from
/// the corresponding [`LayoutCacheEntry`].
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct LayoutCacheEntryId {
    /// Private cache-entry discriminator.
    kind: LayoutCacheEntryKind,
}

impl fmt::Debug for LayoutCacheEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LayoutCacheEntryId(..)")
    }
}

/// Private cache-entry discriminator.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum LayoutCacheEntryKind {
    /// The final layout entry for a full layout pass.
    FinalLayout,
    /// One of the intrinsic-size measurement cache slots.
    Measure {
        /// Private slot index in the node's measurement cache.
        slot: u8,
    },
}

impl LayoutCacheEntryId {
    /// Opaque identity for the final-layout cache entry.
    const FINAL_LAYOUT: Self = Self { kind: LayoutCacheEntryKind::FinalLayout };

    /// Construct an opaque identity for a measurement cache slot.
    fn measure(slot: usize) -> Self {
        debug_assert!(slot < CACHE_SIZE);
        Self { kind: LayoutCacheEntryKind::Measure { slot: slot as u8 } }
    }
}

/// A passive observation of a cache entry Taffy stored or selected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutCacheEntry {
    /// The node whose cache stored or selected this entry.
    node_id: NodeId,
    /// Opaque identity of the stored or selected cache entry.
    entry_id: LayoutCacheEntryId,
    /// The input requested by the layout algorithm for this event.
    requested_input: LayoutInput,
    /// The output returned to the layout algorithm for this event.
    returned_output: LayoutOutput,
}

impl LayoutCacheEntry {
    /// Create a cache-entry event payload.
    pub(crate) fn new(
        node_id: NodeId,
        entry_id: LayoutCacheEntryId,
        requested_input: LayoutInput,
        returned_output: LayoutOutput,
    ) -> Self {
        Self { node_id, entry_id, requested_input, returned_output }
    }

    /// The node whose cache stored or selected this entry.
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Opaque identity of the stored or selected cache entry.
    pub fn entry_id(&self) -> LayoutCacheEntryId {
        self.entry_id
    }

    /// The input requested by the layout algorithm for this event.
    pub fn requested_input(&self) -> LayoutInput {
        self.requested_input
    }

    /// The output returned to the layout algorithm for this event.
    pub fn returned_output(&self) -> LayoutOutput {
        self.returned_output
    }
}

/// A passive observation that Taffy cleared all cache entries for a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutCacheClear {
    /// The node whose cache was cleared.
    node_id: NodeId,
}

impl LayoutCacheClear {
    /// Create a cache-clear event payload.
    pub(crate) fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    /// The node whose cache was cleared.
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

/// Passive layout-cache events emitted by `TaffyTree` compute methods.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum LayoutCacheEvent {
    /// Taffy reused a cached layout result.
    Hit(LayoutCacheEntry),
    /// Taffy stored a newly-computed layout result.
    Stored(LayoutCacheEntry),
    /// Taffy cleared all cache entries for a node during compute.
    Cleared(LayoutCacheClear),
}

/// Cached intermediate layout results
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub(crate) struct CacheEntry<T> {
    /// The initial cached size of the node itself
    known_dimensions: Size<Option<f32>>,
    /// The initial cached size of the parent node, used for percentage resolution
    parent_size: Size<Option<f32>>,
    /// The initial cached available space
    available_space: Size<AvailableSpace>,
    /// The cached size and baselines of the item
    content: T,
}

/// A cache for caching the results of a sizing a Grid Item or Flexbox Item
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cache {
    /// The cache entry for the node's final layout
    final_layout_entry: Option<CacheEntry<LayoutOutput>>,
    /// The cache entries for the node's preliminary size measurements
    measure_entries: [Option<CacheEntry<Size<f32>>>; CACHE_SIZE],
    /// Tracks if all cache entries are empty
    is_empty: bool,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new empty cache
    pub const fn new() -> Self {
        Self { final_layout_entry: None, measure_entries: [None; CACHE_SIZE], is_empty: true }
    }

    /// Return the cache slot to cache the current computed result in
    ///
    /// ## Caching Strategy
    ///
    /// We need multiple cache slots, because a node's size is often queried by it's parent multiple times in the course of the layout
    /// process, and we don't want later results to clobber earlier ones.
    ///
    /// The two variables that we care about when determining cache slot are:
    ///
    ///   - How many "known_dimensions" are set. In the worst case, a node may be called first with neither dimension known, then with one
    ///     dimension known (either width of height - which doesn't matter for our purposes here), and then with both dimensions known.
    ///   - Whether unknown dimensions are being sized under a min-content or a max-content available space constraint (definite available space
    ///     shares a cache slot with max-content because a node will generally be sized under one or the other but not both).
    ///
    /// ## Cache slots:
    ///
    /// - Slot 0: Both known_dimensions were set
    /// - Slots 1-4: 1 of 2 known_dimensions were set and:
    ///   - Slot 1: width but not height known_dimension was set and the other dimension was either a MaxContent or Definite available space constraintraint
    ///   - Slot 2: width but not height known_dimension was set and the other dimension was a MinContent constraint
    ///   - Slot 3: height but not width known_dimension was set and the other dimension was either a MaxContent or Definite available space constraintable space constraint
    ///   - Slot 4: height but not width known_dimension was set and the other dimension was a MinContent constraint
    /// - Slots 5-8: Neither known_dimensions were set and:
    ///   - Slot 5: x-axis available space is MaxContent or Definite and y-axis available space is MaxContent or Definite
    ///   - Slot 6: x-axis available space is MaxContent or Definite and y-axis available space is MinContent
    ///   - Slot 7: x-axis available space is MinContent and y-axis available space is MaxContent or Definite
    ///   - Slot 8: x-axis available space is MinContent and y-axis available space is MinContent
    #[inline]
    fn compute_cache_slot(known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> usize {
        use AvailableSpace::{Definite, MaxContent, MinContent};

        let has_known_width = known_dimensions.width.is_some();
        let has_known_height = known_dimensions.height.is_some();

        // Slot 0: Both known_dimensions were set
        if has_known_width && has_known_height {
            return 0;
        }

        // Slot 1: width but not height known_dimension was set and the other dimension was either a MaxContent or Definite available space constraint
        // Slot 2: width but not height known_dimension was set and the other dimension was a MinContent constraint
        if has_known_width && !has_known_height {
            return 1 + (available_space.height == MinContent) as usize;
        }

        // Slot 3: height but not width known_dimension was set and the other dimension was either a MaxContent or Definite available space constraint
        // Slot 4: height but not width known_dimension was set and the other dimension was a MinContent constraint
        if has_known_height && !has_known_width {
            return 3 + (available_space.width == MinContent) as usize;
        }

        // Slots 5-8: Neither known_dimensions were set and:
        match (available_space.width, available_space.height) {
            // Slot 5: x-axis available space is MaxContent or Definite and y-axis available space is MaxContent or Definite
            (MaxContent | Definite(_), MaxContent | Definite(_)) => 5,
            // Slot 6: x-axis available space is MaxContent or Definite and y-axis available space is MinContent
            (MaxContent | Definite(_), MinContent) => 6,
            // Slot 7: x-axis available space is MinContent and y-axis available space is MaxContent or Definite
            (MinContent, MaxContent | Definite(_)) => 7,
            // Slot 8: x-axis available space is MinContent and y-axis available space is MinContent
            (MinContent, MinContent) => 8,
        }
    }

    #[inline]
    fn nullable_size_matches(cached: Size<Option<f32>>, requested: Size<Option<f32>>) -> bool {
        fn dimension_matches(cached: Option<f32>, requested: Option<f32>) -> bool {
            match (cached, requested) {
                (Some(cached), Some(requested)) => (cached - requested).abs() < f32::EPSILON,
                (None, None) => true,
                _ => false,
            }
        }

        dimension_matches(cached.width, requested.width) && dimension_matches(cached.height, requested.height)
    }

    #[inline]
    fn parent_size_matches<T>(entry: &CacheEntry<T>, input: &LayoutInput) -> bool {
        Self::nullable_size_matches(entry.parent_size, input.parent_size)
    }

    /// Try to retrieve a cached result from the cache
    #[inline]
    pub fn get(&self, input: &LayoutInput) -> Option<LayoutOutput> {
        self.get_with_entry(input).map(|(_, output)| output)
    }

    /// Try to retrieve a cached result and the entry that matched it.
    #[inline]
    pub(crate) fn get_with_entry(&self, input: &LayoutInput) -> Option<(LayoutCacheEntryId, LayoutOutput)> {
        let known_dimensions = input.known_dimensions;
        let available_space = input.available_space;

        match input.run_mode {
            RunMode::PerformLayout => self
                .final_layout_entry
                .filter(|entry| {
                    let cached_size = entry.content.size;
                    Self::parent_size_matches(entry, input)
                        && (known_dimensions.width == entry.known_dimensions.width
                            || known_dimensions.width == Some(cached_size.width))
                        && (known_dimensions.height == entry.known_dimensions.height
                            || known_dimensions.height == Some(cached_size.height))
                        && (known_dimensions.width.is_some()
                            || entry.available_space.width.is_roughly_equal(available_space.width))
                        && (known_dimensions.height.is_some()
                            || entry.available_space.height.is_roughly_equal(available_space.height))
                })
                .map(|e| (LayoutCacheEntryId::FINAL_LAYOUT, e.content)),
            RunMode::ComputeSize => {
                for (slot, entry) in self
                    .measure_entries
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, entry)| entry.as_ref().map(|entry| (slot, entry)))
                {
                    let cached_size = entry.content;

                    if Self::parent_size_matches(entry, input)
                        && (known_dimensions.width == entry.known_dimensions.width
                            || known_dimensions.width == Some(cached_size.width))
                        && (known_dimensions.height == entry.known_dimensions.height
                            || known_dimensions.height == Some(cached_size.height))
                        && (known_dimensions.width.is_some()
                            || entry.available_space.width.is_roughly_equal(available_space.width))
                        && (known_dimensions.height.is_some()
                            || entry.available_space.height.is_roughly_equal(available_space.height))
                    {
                        return Some((LayoutCacheEntryId::measure(slot), LayoutOutput::from_outer_size(cached_size)));
                    }
                }

                None
            }
            RunMode::PerformHiddenLayout => None,
        }
    }

    /// Store a computed size in the cache
    pub fn store(&mut self, input: &LayoutInput, layout_output: LayoutOutput) {
        self.store_with_entry(input, layout_output);
    }

    /// Store a computed size in the cache and return the entry that was written.
    pub(crate) fn store_with_entry(
        &mut self,
        input: &LayoutInput,
        layout_output: LayoutOutput,
    ) -> Option<LayoutCacheEntryId> {
        let known_dimensions = input.known_dimensions;
        let parent_size = input.parent_size;
        let available_space = input.available_space;

        match input.run_mode {
            RunMode::PerformLayout => {
                self.is_empty = false;
                self.final_layout_entry =
                    Some(CacheEntry { known_dimensions, parent_size, available_space, content: layout_output });
                Some(LayoutCacheEntryId::FINAL_LAYOUT)
            }
            RunMode::ComputeSize => {
                self.is_empty = false;
                let cache_slot = Self::compute_cache_slot(known_dimensions, available_space);
                self.measure_entries[cache_slot] =
                    Some(CacheEntry { known_dimensions, parent_size, available_space, content: layout_output.size });
                Some(LayoutCacheEntryId::measure(cache_slot))
            }
            RunMode::PerformHiddenLayout => None,
        }
    }

    /// Clear all cache entries and reports clear operation outcome ([`ClearState`])
    pub fn clear(&mut self) -> ClearState {
        if self.is_empty {
            return ClearState::AlreadyEmpty;
        }
        self.is_empty = true;
        self.final_layout_entry = None;
        self.measure_entries = [None; CACHE_SIZE];
        ClearState::Cleared
    }

    /// Returns true if all cache entries are None, else false
    pub fn is_empty(&self) -> bool {
        self.final_layout_entry.is_none() && !self.measure_entries.iter().any(|entry| entry.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Line;
    use crate::style::AvailableSpace;
    use crate::style_helpers::TaffyMaxContent;
    use crate::tree::{RequestedAxis, SizingMode};

    fn compute_size_input(known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> LayoutInput {
        LayoutInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known_dimensions,
            parent_size: Size::NONE,
            available_space,
            vertical_margins_are_collapsible: Line::FALSE,
        }
    }

    fn input_with_parent_size(run_mode: RunMode, parent_size: Size<Option<f32>>) -> LayoutInput {
        LayoutInput {
            run_mode,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known_dimensions: Size::NONE,
            parent_size,
            available_space: Size::MAX_CONTENT,
            vertical_margins_are_collapsible: Line::FALSE,
        }
    }

    fn stored_output() -> LayoutOutput {
        LayoutOutput::from_outer_size(Size { width: 100.0, height: 0.0 })
    }

    #[test]
    fn equivalent_compute_size_hit_returns_stored_entry_id() {
        let mut cache = Cache::new();
        let stored_input = compute_size_input(Size::NONE, Size::MAX_CONTENT);
        let stored_output = LayoutOutput::from_outer_size(Size { width: 100.0, height: 20.0 });
        let stored_entry_id = cache.store_with_entry(&stored_input, stored_output).unwrap();

        let requested_input = compute_size_input(
            Size { width: Some(100.0), height: None },
            Size { width: AvailableSpace::MaxContent, height: AvailableSpace::MaxContent },
        );
        let (hit_entry_id, hit_output) = cache.get_with_entry(&requested_input).unwrap();

        assert_eq!(hit_entry_id, stored_entry_id);
        assert_eq!(hit_output, LayoutOutput::from_outer_size(stored_output.size));
    }

    #[test]
    fn compute_size_cache_misses_when_parent_size_changes() {
        let mut cache = Cache::new();
        let stored_input = input_with_parent_size(RunMode::ComputeSize, Size { width: Some(100.0), height: Some(0.0) });
        let requested_input =
            input_with_parent_size(RunMode::ComputeSize, Size { width: Some(100.0), height: Some(200.0) });

        cache.store(&stored_input, stored_output());

        assert_eq!(cache.get(&requested_input), None);
    }

    #[test]
    fn final_layout_cache_misses_when_parent_size_changes() {
        let mut cache = Cache::new();
        let stored_input =
            input_with_parent_size(RunMode::PerformLayout, Size { width: Some(100.0), height: Some(0.0) });
        let requested_input =
            input_with_parent_size(RunMode::PerformLayout, Size { width: Some(100.0), height: Some(200.0) });

        cache.store(&stored_input, stored_output());

        assert_eq!(cache.get(&requested_input), None);
    }
}

/// Clear operation outcome. See [`Cache::clear`]
pub enum ClearState {
    /// Cleared some values
    Cleared,
    /// Everything was already cleared
    AlreadyEmpty,
}
