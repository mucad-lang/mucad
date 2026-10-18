// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render cache.

use crate::GeometryOutputs;
use mucad_hash::{HashId, HashMap};

/// An item in the [`RenderCache`].
pub struct RenderCacheItem<T> {
    /// The actual item content.
    content: T,
    /// Number of times this cache item has been accessed successfully.
    hits: u64,
    /// Number of milliseconds this item took to create.
    millis: f64,
    /// Time stamp of the last access to this cache item.
    last_access: u64,
}

impl<T> RenderCacheItem<T> {
    /// Create new cache item.
    pub fn new(content: impl Into<T>, millis: f64, last_access: u64) -> Self {
        Self {
            content: content.into(),
            hits: 1,
            millis,
            last_access,
        }
    }

    fn cost(&self, current_time_stamp: u64) -> f64 {
        // Weighted sum of:
        // - Recency: more recent items are more valuable
        // - Frequency: more frequently accessed items are more valuable
        // - Computation cost: items that are expensive to regenerate are more valuable

        let recency = 1.0 / (1.0 + (current_time_stamp - self.last_access) as f64);
        let frequency = self.hits as f64;
        let computation_cost = self.millis;

        // We can tune these weights.
        let weight_recency = 2.3;
        let weight_frequency = 0.5;
        let weight_computation = 0.2;

        (weight_recency * recency)
            + (weight_frequency * frequency)
            + (weight_computation * computation_cost)
    }
}

/// The [`RenderCache`] owns all geometry created during the render process.
pub struct RenderCache<T = GeometryOutputs> {
    /// Current render cache item stamp.
    current_time_stamp: u64,
    /// Number of cache hits in this cycle.
    hits: u64,
    /// Maximum cost of a cache item before it is removed during garbage collection.
    max_cost: f64,
    /// The actual cache item store.
    items: HashMap<HashId, RenderCacheItem<T>>,
}

impl<T> RenderCache<T> {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            current_time_stamp: 0,
            hits: 0,
            items: HashMap::default(),
            max_cost: std::env::var("MICROCAD_CACHE_MAX_COST")
                .ok()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.2),
        }
    }

    /// Remove old items based on a cost function from the cache.
    pub fn garbage_collection(&mut self) {
        let old_count = self.items.len();
        self.items.retain(|hash, item| {
            let cost = item.cost(self.current_time_stamp);
            let keep = cost > self.max_cost;
            log::trace!(
                "Item {hash} cost = {cost}: {keep}",
                keep = if keep { "🔄" } else { "🗑" }
            );
            keep
        });

        let removed = old_count - self.items.len();
        log::info!(
            "Removed {removed} items from cache. Cache contains {n} items. {hits} cache hits in this cycle.",
            n = self.items.len(),
            hits = self.hits,
        );
        self.current_time_stamp += 1;
        self.hits = 0;
    }

    /// Empty cache entirely.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get geometry output from the cache.
    pub fn get(&mut self, hash: &HashId) -> Option<&T> {
        match self.items.get_mut(hash) {
            Some(item) => {
                item.hits += 1;
                self.hits += 1;
                item.last_access = self.current_time_stamp;
                log::trace!(
                    "Cache hit: {hash}. Cost: {}",
                    item.cost(self.current_time_stamp)
                );
                Some(&item.content)
            }
            _ => None,
        }
    }

    /// Insert geometry output into the cache with pre-estimated cost and return inserted geometry.
    pub fn insert_with_cost(
        &mut self,
        hash: impl Into<HashId>,
        geo: impl Into<T>,
        cost: f64,
    ) -> &T {
        let hash: HashId = hash.into();
        self.items.insert(
            hash,
            RenderCacheItem::new(geo, cost, self.current_time_stamp),
        );
        &self.items.get(&hash).expect("Hash").content
    }
}

impl Default for RenderCache {
    fn default() -> Self {
        Self::new()
    }
}
