## 1. Data Model Changes
- [x] 1.1 Add `cpu_history: VecDeque<f64>` and `memory_history: VecDeque<f64>` to `ContainerStats` in `src/core/types.rs`
- [x] 1.2 Define history buffer size constant (e.g., 20 samples to match bar width)

## 2. Stats Collection
- [x] 2.1 Update `stream_container_stats()` in `src/docker/stats.rs` to push new values to history buffers
- [x] 2.2 Ensure history is capped at max size (pop front when full)

## 3. Sparkline Rendering
- [x] 3.1 Create `create_sparkline()` function in `src/ui/container_list.rs` that converts history to braille characters
- [x] 3.2 Map percentage values to braille vertical heights (⠀⣀⣤⣶⣿)
- [x] 3.3 Replace `create_progress_bar()` calls with `create_sparkline()` for CPU
- [x] 3.4 Replace `create_memory_progress_bar()` calls with `create_sparkline()` for memory

## 4. Tick Markers
- [x] 4.1 Add tick markers every 5 positions using braille dot pattern (⡀)
- [x] 4.2 Create "hole" variants of bars (⢀⢤⢶⢿) for tick positions with data
- [x] 4.3 Implement marching ticks using global wall clock time for synchronization
- [x] 4.4 Ticks only appear in actual data, not in padding
- [x] 4.5 Ticks advance every 2 seconds (half speed) for smoother animation

## 5. Testing
- [x] 5.1 Add unit tests for sparkline rendering with various history patterns
- [x] 5.2 Add unit tests for marching tick behavior
- [x] 5.3 Update existing progress bar tests
- [x] 5.4 Run snapshot tests and accept new baselines with `cargo insta accept`
