use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::app_state::AppState;
use crate::core::types::{
    Container, ContainerKey, ContainerState, ContainerStats, HealthStatus, RenderAction,
    BUCKET_DURATION_SECS, HISTORY_BUFFER_SIZE,
};

/// Returns the current time bucket ID for history synchronization.
/// This aligns with the tick marker calculation in the sparkline renderer.
fn get_current_bucket() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / BUCKET_DURATION_SECS)
        .unwrap_or(0)
}

impl AppState {
    pub(super) fn handle_initial_container_list(
        &mut self,
        host_id: String,
        container_list: Vec<Container>,
    ) -> RenderAction {
        for container in container_list {
            let key = ContainerKey::new(host_id.clone(), container.id.clone());
            self.containers.insert(key.clone(), container);
            self.sorted_container_keys.push(key);
        }

        // Force immediate sort when loading initial container list
        self.force_sort_containers();

        // Select first row if we have containers
        if !self.containers.is_empty() {
            self.table_state.select(Some(0));
        }

        RenderAction::Render // Force draw - table structure changed
    }

    pub(super) fn handle_container_created(&mut self, container: Container) -> RenderAction {
        let key = ContainerKey::new(container.host_id.clone(), container.id.clone());
        self.containers.insert(key.clone(), container);
        self.sorted_container_keys.push(key);

        // Force immediate sort when new container is added
        self.force_sort_containers();

        // Select first row if this is the first container
        if self.containers.len() == 1 {
            self.table_state.select(Some(0));
        }

        RenderAction::Render // Force draw - table structure changed
    }

    pub(super) fn handle_container_destroyed(&mut self, key: ContainerKey) -> RenderAction {
        self.containers.remove(&key);
        self.sorted_container_keys.retain(|k| k != &key);

        // Adjust selection if needed
        let container_count = self.containers.len();
        if container_count == 0 {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected()
            && selected >= container_count
        {
            self.table_state.select(Some(container_count - 1));
        }

        RenderAction::Render // Force draw - table structure changed
    }

    pub(super) fn handle_container_state_changed(
        &mut self,
        key: ContainerKey,
        state: ContainerState,
    ) -> RenderAction {
        if let Some(container) = self.containers.get_mut(&key) {
            container.state = state;
            return RenderAction::Render; // Force draw - state changed
        }
        RenderAction::None
    }

    pub(super) fn handle_container_stat(
        &mut self,
        key: ContainerKey,
        mut stats: ContainerStats,
    ) -> RenderAction {
        if let Some(container) = self.containers.get_mut(&key) {
            // Preserve existing history
            let mut cpu_history = std::mem::take(&mut container.stats.cpu_history);
            let mut memory_history = std::mem::take(&mut container.stats.memory_history);
            let last_bucket = container.stats.last_history_bucket;

            // Get current time bucket (synchronized with tick markers)
            let current_bucket = get_current_bucket();

            // Only add to history if we've moved to a new time bucket
            // This ensures history samples align with tick marker intervals
            if current_bucket > last_bucket {
                cpu_history.push_back(stats.cpu);
                memory_history.push_back(stats.memory);

                // Cap history at max size
                while cpu_history.len() > HISTORY_BUFFER_SIZE {
                    cpu_history.pop_front();
                }
                while memory_history.len() > HISTORY_BUFFER_SIZE {
                    memory_history.pop_front();
                }

                stats.last_history_bucket = current_bucket;
            } else {
                // Keep the existing bucket ID if we haven't moved to a new bucket
                stats.last_history_bucket = last_bucket;
            }

            // Assign history to the new stats
            stats.cpu_history = cpu_history;
            stats.memory_history = memory_history;

            // Always update displayed values (responsive current values)
            container.stats = stats;
        }
        RenderAction::None // No force draw - just stats update
    }

    pub(super) fn handle_container_health_changed(
        &mut self,
        key: ContainerKey,
        health: HealthStatus,
    ) -> RenderAction {
        if let Some(container) = self.containers.get_mut(&key) {
            container.health = Some(health);
        }
        RenderAction::Render // Force draw - health status changed (visible in UI)
    }
}
