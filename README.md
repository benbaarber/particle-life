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

`cd` into wgpu or macroquad directory and run `cargo run -r`

In wgpu, in main.rs, you can adjust the following values to change the sim behavior:
- aoe: area of effect, lower values are more localized and smaller effects, and
generally better performance since the spatial binning grid size is derived
from the aoe. higher values are more chaotic and worse performance, so lower
the particle count if using a high aoe.
- damping: velocity damping
- num_cultures: number of different particle groups
- culture_size: particles per culture

Keybinds:
- q: quit
- r: reset (mq only)

