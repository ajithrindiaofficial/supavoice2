# Supavoice Implementation Plan

## 🎯 Project Overview
Build a voice transcription and AI-powered formatting app with **offline-first** architecture, supporting both local models and cloud APIs (OpenAI).

---

## 📋 Detailed Implementation Plan

### **Phase 1: Foundation & Infrastructure** (Week 1-2)

#### 1.1 Project Cleanup & Setup
- Update `package.json` name from "ajith-macos-starter" to "supavoice"
- Update database migration reference in [main.rs:269](supavoice/src-tauri/src/main.rs#L269) from `"ajith-macos-starter.db"` to `"supavoice.db"`
- Update tray tooltip in [main.rs:280](supavoice/src-tauri/src/main.rs#L280)
- Remove existing time tracking UI/database schema (clean slate for transcription app)

#### 1.2 Rust Dependencies Architecture
Add to `Cargo.toml`:
```toml
# Core async & HTTP
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["stream", "json"] }

# Error handling & serialization
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Crypto & file management
sha2 = "0.10"
directories = "5.0"

# Offline models (add later in Phase 2)
# whisper-rs = { git = "https://github.com/tazz4843/whisper-rs" }
# llama-cpp-rs = { git = "https://github.com/rustformers/llama-cpp-rs" }

[target.'cfg(target_os = "macos")'.dependencies]
# Uncomment in Phase 3 for Metal GPU acceleration
# whisper-rs = { git = "https://github.com/tazz4843/whisper-rs", features = ["metal"] }
```

#### 1.3 Frontend Dependencies
```bash
npm install zustand @tanstack/react-query
```
- **zustand**: Lightweight state management for settings/mode
- **react-query**: Handle async operations (downloads, API calls)

---

### **Phase 2: Model Registry & Download Manager** (Week 2-3)

#### 2.1 Rust Backend - Model Registry
Create `src-tauri/src/models/` module structure:

**File: `models/types.rs`**
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModelKind {
    Whisper,
    LLM,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModelStatus {
    NotInstalled,
    Downloading { progress: f32, bytes: u64, total: u64 },
    Installed,
    Failed { error: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelRecord {
    pub id: String,
    pub name: String,
    pub kind: ModelKind,
    pub size_mb: u32,
    pub download_url: String,
    pub checksum: String, // SHA-256
    pub status: ModelStatus,
    pub path: Option<PathBuf>,
}
```

**File: `models/registry.rs`**
- In-memory `HashMap<String, ModelRecord>` wrapped in `Arc<Mutex<_>>`
- Persist to `AppData/models/registry.json`
- Hardcode initial catalog:
  ```rust
  // Whisper models
  - small.en (466 MB) [Primary - best balance]
  - base.en (142 MB) [Minimal footprint option]

  // LLM models
  - Gemma-2-2B-Instruct Q4_K_M (1.71 GB) [Primary - excellent instruction following]
  - Qwen2-1.5B-Instruct Q4_K_M (986 MB) [Lightweight alternative]
  ```

**File: `models/downloader.rs`**
- Streaming download with `reqwest`
- Write to `.part` file with atomic rename on success
- SHA-256 verification
- Emit Tauri events: `download_progress`, `download_complete`, `download_failed`
- Support cancellation via `CancellationToken`

#### 2.2 Tauri Commands
```rust
#[tauri::command]
async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelRecord>, String>

#[tauri::command]
async fn start_download(state: State<'_, AppState>, model_id: String) -> Result<(), String>

#[tauri::command]
async fn cancel_download(state: State<'_, AppState>, model_id: String) -> Result<(), String>

#[tauri::command]
async fn delete_model(state: State<'_, AppState>, model_id: String) -> Result<(), String>

#[tauri::command]
async fn get_disk_space() -> Result<u64, String> // Free disk space check
```

#### 2.3 Frontend - Settings UI
Create `src/pages/Settings.tsx`:
- **Tab 1: Models**
  - Table with columns: Name, Type, Size, Status, Actions
  - Actions: Download/Cancel/Delete buttons
  - Real-time progress bars (listen to `download_progress` events)
  - Disk space warning if insufficient

- **Tab 2: API Keys** (for Phase 4)
  - OpenAI API key input (stored securely in Tauri state)
  - Test connection button

- **Tab 3: Preferences**
  - Mode toggle: Offline / Online
  - GPU acceleration toggle (macOS Metal)

---

### **Phase 3: Offline Transcription (Whisper)** (Week 3-4)

#### 3.1 Audio Capture
Add dependency:
```toml
cpal = "0.15" # Cross-platform audio I/O
hound = "3.5" # WAV file encoding
```

**File: `audio/recorder.rs`**
- Record audio from system microphone
- Output PCM16 format @ 16kHz sample rate (Whisper requirement)
- Push-to-talk or auto-stop with VAD (optional for v1)

#### 3.2 Whisper Integration
Add dependency (with Metal support on macOS):
```toml
# For non-macOS platforms
whisper-rs = { git = "https://github.com/tazz4843/whisper-rs" }

[target.'cfg(target_os = "macos")'.dependencies]
whisper-rs = { git = "https://github.com/tazz4843/whisper-rs", features = ["metal"] }
```

**Why whisper.cpp (via whisper-rs)?**
- Native Metal GPU acceleration on macOS (2-3x faster than CPU)
- Mature Rust bindings, easier cross-platform packaging
- Good enough performance for MVP (30-sec audio → ~8-10 sec on M1 with Metal)
- Can add Faster-Whisper backend later for CPU-optimized workloads

**File: `transcription/whisper.rs`**
```rust
#[tauri::command]
async fn transcribe_audio(
    state: State<'_, AppState>,
    audio_path: String,
) -> Result<TranscriptionResult, String> {
    // Load model from AppData/models/whisper/small.en/
    // Process audio with whisper-rs
    // Return segments with timestamps
}
```

**Output format:**
```json
{
  "text": "Full transcript here",
  "segments": [
    { "start": 0.0, "end": 3.2, "text": "Hello world" }
  ]
}
```

#### 3.3 Frontend - Recording UI
Update `src/App.tsx` (main overlay window):
- Microphone icon button (push-to-hold or click-to-start)
- Waveform visualization during recording
- Transcript display area
- "Format" button to trigger LLM processing

---

### **Phase 4: Offline Formatting (LLM)** (Week 4-5)

#### 4.1 llama.cpp Integration (Gemma-2-2B)
Add dependency:
```toml
llama-cpp-rs = { git = "https://github.com/rustformers/llama-cpp-rs" }
```

**File: `formatting/llm.rs`**
```rust
#[tauri::command]
async fn format_transcript(
    state: State<'_, AppState>,
    transcript: String,
    template: String, // "email", "linkedin", "notes"
) -> Result<FormattedOutput, String> {
    // Load Gemma-2-2B-Instruct GGUF model
    // Use JSON-schema constrained decoding
    // Return structured output
}
```

**Why Gemma-2-2B over Llama?**
- Smaller size: 1.71 GB vs 5.7 GB (Llama-3.1-8B)
- Faster inference: 2B params = quicker response for short transcripts
- Strong instruction following (Google-trained for chat/instruction tasks)
- Better GGUF support and more permissive license (Apache 2.0)

**JSON Schema for constrained output:**
```json
{
  "type": "object",
  "properties": {
    "title": { "type": "string" },
    "body": { "type": "string" },
    "cta": { "type": "string" }
  },
  "required": ["body"]
}
```

#### 4.2 Template System
Create `templates/` folder with predefined prompts:
- **Email**: "Rewrite this transcript as a professional email..."
- **LinkedIn Post**: "Transform this into an engaging LinkedIn post..."
- **Clean Notes**: "Format this as clear bullet points..."

Store in `AppData/templates.json` (user-editable later)

#### 4.3 Frontend - Formatting UI
Add to transcript view:
- Template selector dropdown
- "Format" button → triggers `format_transcript` command
- Side-by-side view: Original | Formatted
- Copy to clipboard button

---

### **Phase 5: Online Mode (OpenAI API)** (Week 5-6)

#### 5.1 API Adapter Pattern
**File: `providers/mod.rs`**
```rust
#[async_trait]
trait TranscriptionProvider {
    async fn transcribe(&self, audio: &[u8]) -> Result<String>;
}

#[async_trait]
trait FormattingProvider {
    async fn format(&self, text: &str, template: &str) -> Result<String>;
}
```

**Implementations:**
- `providers/openai.rs` - Whisper API + GPT-4
- `providers/local.rs` - whisper-rs + Gemma-2-2B via llama.cpp

#### 5.2 Mode Switching
```rust
#[tauri::command]
fn set_mode(state: State<'_, AppState>, offline: bool) -> Result<(), String>
```

Frontend logic:
- If offline mode + model missing → show "Install Models" modal
- If online mode + no API key → show "Enter API Key" modal
- Graceful fallback: offline fails → prompt to use online

---

### **Phase 6: Polish & Optimization** (Week 6-7)

#### 6.1 Performance Optimizations
- **Metal/GPU Support**: Already enabled in Phase 3 for macOS
- **Model Quantization**: Already using Q4_K_M variants for optimal size/quality
- **Streaming**: Process audio chunks in real-time (advanced)
- **Optional**: Add Faster-Whisper backend for CPU-optimized transcription on Windows/Linux
  - Uses CTranslate2 with int8 quantization (2-4x faster CPU, ~40% less memory)
  - Trade-off: More complex packaging, less mature Rust bindings

#### 6.2 UX Improvements
- First-run onboarding flow
- Microphone permission check
- Progress indicators for long operations
- Error handling with user-friendly messages
- Keyboard shortcuts (existing global shortcut plugin)

#### 6.3 Testing
- Unit tests for download manager (checksum, resume)
- Integration tests for transcription pipeline
- Manual testing matrix: macOS (M1, Intel), Windows, Linux

---

## 🗂️ Final File Structure

```
src-tauri/src/
├── main.rs
├── models/
│   ├── mod.rs
│   ├── types.rs
│   ├── registry.rs
│   └── downloader.rs
├── audio/
│   ├── mod.rs
│   └── recorder.rs
├── transcription/
│   ├── mod.rs
│   └── whisper.rs
├── formatting/
│   ├── mod.rs
│   └── llm.rs
└── providers/
    ├── mod.rs
    ├── openai.rs
    └── local.rs

src/
├── App.tsx (main recording UI)
├── pages/
│   └── Settings.tsx
├── components/
│   ├── AudioRecorder.tsx
│   ├── TranscriptView.tsx
│   ├── FormatSelector.tsx
│   └── ModelManager.tsx
└── hooks/
    ├── useDownload.ts
    └── useTranscription.ts
```

---

## ⚡ Quick Wins to Start

1. **Week 1 Goal**: Get model download working with progress UI
2. **Week 2 Goal**: Integrate whisper-rs, transcribe a test WAV file
3. **Week 3 Goal**: Add microphone recording + basic UI
4. **Week 4 Goal**: Integrate llama.cpp, format first transcript

---

## 🔑 Key Decisions

| Decision | Rationale |
|----------|-----------|
| **Offline-first** | Privacy, speed, works without internet |
| **whisper.cpp (via whisper-rs)** | Metal GPU support on macOS, simpler packaging, mature Rust bindings |
| **Gemma-2-2B-Instruct** | Optimal size/quality for formatting (1.71 GB), fast inference, strong instruction following |
| **llama.cpp** | Best Rust bindings, JSON-schema constrained decoding support |
| **JSON-schema** | Guaranteed parseable output, no regex hacks |
| **Tauri events** | Real-time progress updates to UI |
| **Defer Faster-Whisper** | Keep MVP simple; add as optional backend later for CPU optimization |

---

## 🚀 MVP Scope (Ship in 4 weeks)

**Must-have:**
- ✅ Download & manage 1 Whisper model (small.en with Metal support)
- ✅ Download & manage 1 LLM (Gemma-2-2B-Instruct)
- ✅ Record audio from mic
- ✅ Transcribe offline (Metal GPU acceleration on macOS)
- ✅ Format with 2 templates (Email, Notes)
- ✅ Copy to clipboard

**Nice-to-have (defer to v1.1):**
- VAD for auto-segmentation
- Online mode with OpenAI
- Custom templates
- Multi-speaker diarization
- Resume interrupted downloads
- Faster-Whisper backend option (CPU-optimized for Windows/Linux)

---

## 📚 Model Details Reference

### Whisper Models
- **small.en** (English only): ~466 MB disk, ~852 MB RAM - Best balance for desktop
- **base.en** (English only): ~142 MB disk - Minimal footprint, accuracy step down
- **small** (multilingual): ~466 MB disk - Same size as small.en, slightly slower

### LLM Models
- **Gemma-2-2B-Instruct** (GGUF Q4_K_M): ~1.71 GB - **PRIMARY CHOICE** - Excellent instruction following, fast inference, optimal for MVP
- **Qwen2-1.5B-Instruct** (GGUF Q4_K_M): ~986 MB - Lightweight alternative for lower-end hardware

### Technical Notes
- **GGUF**: Model file format used by llama.cpp
- **Quantization (Q4/Q5)**: Compresses model weights to fewer bits (4-5 bits) to shrink size with minimal quality loss
- **whisper.cpp**: Fast C/C++ port of Whisper (runs on CPU/GPU locally)
- **Metal**: Apple's GPU framework - enables hardware acceleration on macOS
- **Faster-Whisper**: Optimized Whisper using CTranslate2 (int8 quantization, 2-4x CPU speedup)
- **Grammar/JSON-schema**: Constrain LLM outputs to valid structures (e.g., JSON)
- **VAD**: Voice Activity Detection - detects where speech starts/ends
- **ASR**: Automatic Speech Recognition

### Performance Estimates (30-second audio transcription)
| Setup | Processing Time | Memory Usage |
|-------|----------------|--------------|
| whisper.cpp CPU-only | ~25-30 sec | 1.2 GB |
| whisper.cpp Metal (M1 Mac) | ~8-10 sec | 900 MB |
| Faster-Whisper int8 CPU | ~12-15 sec | 700 MB |

**Recommendation:** Use whisper.cpp with Metal for MVP. Consider adding Faster-Whisper in v1.1 for users on Windows/Linux who need better CPU performance.
