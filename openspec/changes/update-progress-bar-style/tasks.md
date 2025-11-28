## 1. Data Model Changes
- [ ] 1.1 Add `cpu_history: VecDeque<f64>` and `memory_history: VecDeque<f64>` to `ContainerStats` in `src/core/types.rs`
- [ ] 1.2 Define history buffer size constant (e.g., 20 samples to match bar width)

## 2. Stats Collection
- [ ] 2.1 Update `stream_container_stats()` in `src/docker/stats.rs` to push new values to history buffers
- [ ] 2.2 Ensure history is capped at max size (pop front when full)

## 3. Sparkline Rendering
- [ ] 3.1 Create `create_sparkline()` function in `src/ui/container_list.rs` that converts history to braille characters
- [ ] 3.2 Map percentage values to braille vertical heights (⠀⣀⣤⣶⣿)
- [ ] 3.3 Replace `create_progress_bar()` calls with `create_sparkline()` for CPU
- [ ] 3.4 Replace `create_memory_progress_bar()` calls with `create_sparkline()` for memory

## 4. Testing
- [ ] 4.1 Add unit tests for sparkline rendering with various history patterns
- [ ] 4.2 Update existing progress bar tests
- [ ] 4.3 Run snapshot tests and accept new baselines with `cargo insta accept`
