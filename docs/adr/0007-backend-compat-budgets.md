# ADR 0007: Backend Compatibility Contract and Performance Budgets

## Status
Proposed

## Context
As Sermon matures into a high-fidelity audio player, we need to formalize what we support and how we measure "performance". Without a compatibility contract, we risk regression in format support. Without performance budgets, "optimization" is a vague goal rather than a measurable engineering target. Furthermore, high-fidelity playback requires strict realtime guarantees to prevent audible artifacts.

## Decision
We will define and adhere to a strict Backend Compatibility Contract and Performance Budgets.

### 1. Compatibility Matrix

#### Containers & Demuxers
| Format | Container | Support Level | Source |
| :--- | :--- | :--- | :--- |
| **FLAC** | Native (.flac) | Full | Symphonia |
| **WAV** | RIFF (.wav) | Full | Symphonia |
| **MP3** | MP3 (.mp3) | Full | Symphonia |
| **AAC** | MP4, ADTS (.m4a, .aac) | Full | Symphonia |
| **ALAC** | MP4 (.m4a) | Full | Symphonia |
| **Vorbis** | Ogg (.ogg) | Full | Symphonia |
| **AIFF** | AIFF (.aif, .aiff) | Full | Symphonia |
| **MKV/WebM** | Matroska (.mkv, .webm) | Full | Symphonia |
| **DSF** | Sony DSD (.dsf) | Full | `dsd_decode.rs` |
| **DFF** | Philips DSD (.dff) | Full (No DST) | `dsd_decode.rs` |

#### Codecs
| Codec | Support Level | Bit-Depth Support |
| :--- | :--- | :--- |
| **PCM** | Full | 8, 16, 24, 32-bit (Int/Float) |
| **FLAC** | Full | 16, 24-bit |
| **MP3** | Full | Layer 1, 2, 3 |
| **AAC** | Full | LC |
| **Vorbis** | Full | All |
| **DSD** | Full | DSD64, DSD128, DSD256 (DoP/Native) |

#### Output Backends
| Backend | Mode | Support Level | Target Use Case |
| :--- | :--- | :--- | :--- |
| **WASAPI** | Shared | Full | General desktop use |
| **WASAPI** | Exclusive | Full (Bit-Perfect) | Audiophile listening |
| **ASIO** | Native | Planned | Professional interfaces |

---

### 2. Performance Budgets
*Targets based on Baseline Hardware: Ryzen 7 9800X3D, 32GB DDR5, SSD.*

| Metric | Target | Hard Limit |
| :--- | :--- | :--- |
| **App Startup Time** | < 1.0s | 2.0s |
| **Playback Start Latency** | < 100ms | 250ms |
| **Seek Latency** | < 50ms | 150ms |
| **Max Underruns** | 0 per hour | 1 per day |
| **Idle CPU Usage** | < 0.1% | 0.5% |
| **Playback CPU Usage** | < 1.0% (FLAC) | 2.0% |
| **Memory Footprint** | < 150MB | 512MB |

---

### 3. Realtime Guardrails
To ensure "Bit-Perfect" and glitch-free playback, the following operations are **STRICTLY BANNED** on the audio callback thread (hot path):

1.  **Blocking I/O**: No file reads, no network calls.
2.  **Database Access**: No SQLite queries or transactions.
3.  **Allocations**: No `Box::new`, `Vec::push`, or `String` allocations. Use pre-allocated ring buffers.
4.  **Logging**: No `println!` or synchronous `tracing` calls. Use asynchronous logging or atomic status flags.
5.  **Locking**: No heavy mutexes. Use `parking_lot` for fast locks or preferably lock-free primitives (e.g., `crossbeam-channel`, `ringbuf`).

---

### 4. Audio Correctness
*   **Bit-Perfect Requirement**: In Exclusive/ASIO modes, the output sample stream must match the source stream exactly (no resampling, no dither, no volume scaling) unless explicitly requested by the user.
*   **Sample Rate Negotiation**: The engine must attempt to set the hardware sample rate to match the source file. If the hardware does not support the source rate, high-quality resampling (via `rubato`) is the only permitted fallback.

## Consequences
*   **Developer Discipline**: Developers must use pre-allocated buffers and lock-free communication between the decode thread and output thread.
*   **Testing Rigor**: Performance regressions must be treated as bugs.
*   **Hardware Transparency**: Users will have clear expectations of what files will play and how the system will perform.
