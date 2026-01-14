Overall: **yes** — the milestone sequencing is coherent, the scope boundaries are crisp (especially “no other streaming/accounts”), and you’ve already baked in the two things that usually bite vibe-coded desktop apps: **deterministic UI snapshots** and **explicit acceptance criteria**.

Before you feed this to Prometheus, here are the main “add/clarify now so you don’t regret it later” items, with the biggest wins up top.

---

## 1) Add one cross-cutting spec: “Play session + time accounting” (drives history _and_ Last.fm)

Right now, play history + scrobbling can get subtly wrong if you don’t define “played_ms” precisely (pause, seek, track restart, crossfade later, etc.).

Add a small doc section (or ADR) that defines:

- **Play session identity:** `play_id` (UUID/ULID) created when a track starts.
    
- **Time accounting rules:**
    
    - `played_ms` accumulates only while the engine is actually rendering audio (exclude pause).
        
    - Seeking **doesn’t** add time by itself; only playback after the seek counts.
        
    - If the user restarts a track, that’s a **new** play session.
        
- **Single source of truth:** the audio engine emits events like:
    
    - `PlaybackStarted { play_id, track_id, started_at }`
        
    - `PlaybackProgress { play_id, played_ms }` (or periodic tick)
        
    - `PlaybackStopped { play_id, reason }`
        

This prevents your UI, DB, and scrobbler from all trying to “estimate” time in slightly different ways (which is where scrobble bugs come from).

It also slots perfectly into your existing “Now Playing bar becomes functional” milestone.

---

## 2) Gapless playback: either commit to it (same-format) or explicitly defer it

Audiophile users notice gaps instantly, and “scan → click → play” apps often accidentally bake in gaps early.

You don’t have to build full “gapless across format changes” (exclusive mode re-init makes that inherently hard), but you should decide:

- **Gapless for same sample rate/bit depth** tracks in the queue: **in-scope** soon-ish, or **deferred**.
    
- If deferred, add it as an explicit non-scope item so expectations are managed.
    

This is an architecture decision because it affects queue prebuffering and output-stream lifetime (especially once you hit WASAPI Exclusive with per-track format switching).

---

## 3) Important hidden interaction: “safe-write tagging” can break your “stable file identity” model

You’re correctly planning:

- **Stable file identity** on Windows using Volume Serial + File ID for rename/move tolerance.
    
- **Atomic safe write** for tags (temp file → fsync → atomic replace).
    

On Windows/NTFS, an atomic replace often results in a _new_ file identity (because it’s effectively a new file). If your `tracks` table identity depends on file ID, a tag edit can look like “old track deleted, new track appeared” → duplicates, broken play history joins, etc.

What to add to the plan:

- After tag write completes, **re-read the file identity** and update the existing `tracks` row (don’t create a new track).
    
- Make `track_id` an **app-stable primary key** (never changes), with file identity as mutable fields.
    

This one change prevents a ton of downstream weirdness (history, artwork mappings, playlists later, etc.).

---

## 4) Last.fm: a few details worth pinning down before implementation

Your milestone 10 scope is solid (auth, now playing, scrobble thresholds, retry queue). The key additions I’d make are:

### 4.1 Scrobble timing rules (make them explicit in the doc)

Last.fm’s scrobble guideline is straightforward:

- Track must be **> 30 seconds**
    
- Must be played **≥ 50%** of duration **or 4 minutes**, whichever comes first ([Last.fm](https://www.last.fm/api/scrobbling?utm_source=chatgpt.com "API Docs | Last.fm"))
    

Make sure your plan states:

- Whether “played time” excludes pauses (I strongly recommend yes — ties back to the “play session” spec above).
    
- What happens if duration is unknown (fallback to 4 minutes is reasonable, but document it).
    

### 4.2 Batch + retry design (small win, big reliability)

Last.fm allows **batch scrobbles up to 50 per request**. ([Last.fm](https://www.last.fm/api/scrobbling?utm_source=chatgpt.com "API Docs | Last.fm"))  
If you’re already building an offline retry queue, batching gives:

- fewer API calls (lower rate-limit risk)
    
- faster catch-up after offline
    

### 4.3 Auth + signature mechanics (so the agent doesn’t “approximate” it)

Authenticated calls require:

- `api_key`
    
- `sk` (session key)
    
- `api_sig` (signature) ([Last.fm](https://www.last.fm/api/show/track.updateNowPlaying?utm_source=chatgpt.com "API Docs | Last.fm"))
    

And the signature construction is:

- sort params alphabetically
    
- concatenate as `<name><value>`
    
- append shared secret
    
- MD5 the resulting string ([Last.fm](https://www.last.fm/api/webauth "API Docs | Last.fm"))
    

Also: session keys are effectively long-lived, but users can revoke them; store the key securely and handle invalidation gracefully. ([Last.fm](https://www.last.fm/api/webauth "API Docs | Last.fm"))

### 4.4 Secret/session storage: don’t put it in plaintext SQLite

A Last.fm **session key is sensitive**, and the shared secret is even worse.

Given your “minimal deps” requirement, you have a few options:

- **Best UX/security:** store session key in OS credential store (Windows Credential Manager). The Rust `keyring` crate explicitly uses Windows Generic Credentials. ([Docs.rs](https://docs.rs/keyring/latest/x86_64-pc-windows-msvc/keyring/windows/index.html?utm_source=chatgpt.com "keyring::windows - Rust - Docs.rs"))
    
- **Windows-first minimal:** use DPAPI to encrypt before storing in SQLite/config (DPAPI decryptable only by same user context). ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata?utm_source=chatgpt.com "CryptProtectData function (dpapi.h) - Win32 apps | Microsoft Learn"))
    
- **Tauri ecosystem option:** Stronghold plugin provides secure storage, but it’s heavier (still reasonable if you want cross-platform later). ([Tauri](https://v2.tauri.app/plugin/stronghold/?utm_source=chatgpt.com "Stronghold - Tauri"))
    

Also note: Tauri’s Store plugin stores **unencrypted** values on disk, so it’s not appropriate for secrets. ([tauri-apps.github.io](https://tauri-apps.github.io/tauri-plugin-store/?utm_source=chatgpt.com "tauri-plugin-store-api - GitHub Pages"))

### 4.5 One “gotcha” to consider: Last.fm API Terms / commercial intent

If you’re planning to eventually charge money, review Last.fm’s API terms early — their terms describe non-commercial use permissions and that rate limits are enforced at their discretion. ([Last.fm](https://www.last.fm/api/tos?utm_source=chatgpt.com "API Terms of Service - Last.fm"))  
This is more “risk register” than “implementation,” but it’s worth flagging now.

---

## 5) `play_history` table: add a couple fields/indexes so it stays useful

Your proposed schema is a good start:

- `play_history(track_id, started_at, played_ms, scrobbled_at, lastfm_status)`
    

Before you lock it in, I’d add:

- `id` (primary key) or `play_id` (UUID) — makes debugging and idempotency easier.
    
- `ended_at` (optional) and/or `stop_reason` (completed/skipped/error).
    
- `lastfm_attempts`, `lastfm_last_error` (nullable, short text), `lastfm_next_retry_at` — huge for “Scrobble Diagnostics” support.
    

And index:

- `(started_at DESC)` for “Recently played”
    
- `(lastfm_status, started_at)` for retry worker queries
    

This keeps the history view fast and makes offline retry robust without inventing a second queue table.

---

## 6) Windows productization: a few practical clarifications to bake into milestone 11

### 6.1 Installer choice is correctly framed — just document build constraints

Tauri v2 docs state Windows distribution can be:

- **MSI via WiX Toolset v3**
    
- or **NSIS setup exe** ([Tauri](https://v2.tauri.app/distribute/windows-installer/?utm_source=chatgpt.com "Windows Installer - Tauri"))
    

Also: MSI creation is Windows-only because WiX runs on Windows. ([Tauri](https://v2.tauri.app/distribute/windows-installer/?utm_source=chatgpt.com "Windows Installer - Tauri"))  
So your `/docs/release.md` should explicitly note “MSI build must run on Windows” (helps CI planning).

### 6.2 Updater: key management is the real “architecture” part

Tauri updater **requires signatures**; it can’t be disabled. You need a public key in config and a private key to sign artifacts — and if you lose the private key, you can’t ship future updates to existing installs. ([Tauri](https://v2.tauri.app/plugin/updater/?utm_source=chatgpt.com "Updater - Tauri"))

So I’d add to your ADR:

- Where keys live (local dev vs CI secret store)
    
- How you rotate keys (answer: you basically can’t without breaking old installs, so treat it like a root key)
    

### 6.3 WebView2 install mode impacts your installer UX

Tauri supports multiple WebView2 strategies (download bootstrapper, embed bootstrapper, offline installer, fixed runtime, or skip). ([Tauri](https://v2.tauri.app/distribute/windows-installer/?utm_source=chatgpt.com "Windows Installer - Tauri"))  
Decide now which one you want for “minimal Windows product”:

- Default (download bootstrapper): smallest installer, needs internet.
    
- Offline installer/fixed runtime: bigger installer, works offline.
    

### 6.4 Media keys: choose “native API via windows crate” or a plugin

You can implement SMTC directly with the Rust `windows` crate (SystemMediaTransportControls). ([Microsoft GitHub](https://microsoft.github.io/windows-docs-rs/doc/windows/Media/struct.SystemMediaTransportControls.html?utm_source=chatgpt.com "SystemMediaTransportControls in windows::Media - Rust"))  
Or adopt a Tauri plugin such as `tauri-plugin-media` (third-party) that wraps system media controls cross-platform. ([GitHub](https://github.com/Taiizor/tauri-plugin-media?utm_source=chatgpt.com "Taiizor/tauri-plugin-media - GitHub"))

Given your “minimal deps” stance and that you’re already Windows-first (and already using `windows` for audio), I’d lean **native SMTC via `windows` crate** unless you want cross-platform soon.

### 6.5 Add “single instance” to avoid double-playback + weird file-open behavior

On Windows, double-clicking the app or opening files can spawn multiple instances unless you prevent it. There’s an official Tauri Single Instance plugin. ([Tauri](https://v2.tauri.app/plugin/single-instance/?utm_source=chatgpt.com "Single Instance - Tauri"))  
This is small but very “product polish.”

---

## 7) Docs tweaks to make Prometheus (or any agent) perform better

Before you feed these in, add a tiny bit of structure that helps an agent avoid hallucinating interfaces:

1. **One-page architecture map**
    
    - crates/modules and what owns what (scanner, db, audio engine, last.fm service, UI).
        
2. **Explicit Tauri command/event contract**
    
    - command names + payloads + event channels.
        
3. **Glossary**
    
    - “bit-perfect,” “strict vs compatibility,” “play session,” “scrobble eligible,” etc.
        
4. **Redaction policy for diagnostics bundle**
    
    - what’s included by default vs opt-in (paths, track names, etc.).
        

You already have the deterministic UI vision loop and snapshot artifacts discipline, which is a huge advantage for “agentic coding.”

---

## 8) Minor but high-leverage adds (quick list)

- **History retention policy** (e.g., cap at 10k plays or 90 days) + “Clear history” button.
    
- **DB pragmas** documented (WAL mode, synchronous level) once you start measuring perf (ties into your perf-baseline goals).
    
- **“Bit-perfect: Yes/No (why)”** indicator is already suggested — I’d make it non-optional because it reduces support tickets dramatically.
    
- For artwork providers: add a short “network calls & attribution” note (you already plan `/docs/artwork.md`).
    
- For ASIO: keep the licensing checklist prominent (you already planned this; good).
    

---

If you make only **three** changes before feeding this to Prometheus, I’d do:

1. the **play session/time accounting** spec,
    
2. the **tagging vs file identity** interaction note,
    
3. the **Last.fm secret/session storage** decision.
    

Those three prevent the most expensive-to-unwind mistakes later.