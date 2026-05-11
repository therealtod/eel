# Rust Rules

- No unwrap/expect in production code
- Prefer borrowing over cloning
- Avoid Arc<Mutex<T>> unless required
- Benchmark before micro-optimizing
- Prefer explicit trait bounds
- 