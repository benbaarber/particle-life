# Particle Life
<img width="635" height="660" alt="image" src="https://github.com/user-attachments/assets/40683aaa-f0a4-440f-999f-3479e0bdbed1" />

## Benchmarks

| Particles | Steps      | Naive CPU (ms) | Barnes–Hut CPU (ms) | Naive GPU (ms) |
| --------- | ---------- | -------------- | ------------------- | -------------- |
| 1 000     | 100        | 1.996          | 1.701               | 0.275          |
| 10 000    | 100        | 117.677        | 22.241              | 1.988          |
| 50 000    | 10 / 100 * | 3231.8         | 113.246             | 23.557         |
| 200 000   | 100        | —              | 826.473             | 350.294        |

* Naive CPU ran for 10 steps at 50 000 particles; others ran for 100.

## Running

Run wgpu or macroquad version with `cargo run -r --bin (wg|mq)`

Keybinds:
- q: quit
- r: reset (mq only)

