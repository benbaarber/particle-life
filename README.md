# Particle Life

<img width="700" height="565" alt="image" src="https://github.com/user-attachments/assets/ff973efe-778b-48b5-8e09-1316f6572d73" />

## Benchmarks

| Particles | Steps      | Naive CPU (ms) | Barnes–Hut CPU (ms) | Naive GPU (ms) |
| --------- | ---------- | -------------- | ------------------- | -------------- |
| 1 000     | 100        | 1.996          | 1.701               | 0.275          |
| 10 000    | 100        | 117.677        | 22.241              | 1.988          |
| 50 000    | 10 / 100 * | 3231.8         | 113.246             | 23.557         |
| 200 000   | 100        | —              | 826.473             | 350.294        |

* Naive CPU ran for 10 steps at 50 000 particles; others ran for 100.

## Running

Run with `cargo run -r`

Keybinds:
- q: quit
- r: reset

