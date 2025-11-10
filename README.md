# Notes
A program that allows you to record any lessons and later transcribe and summarize them automatically.

## Frontend
Build with tauri and leptos. Only record and sends the reccordings to the backend.

## Backend
Build with leptos, whisper-rs and ollama. \
Transcribes and summarizes the lectures.

## Demo
https://hc-cdn.hel1.your-objectstorage.com/s/v3/aeb94e7f22188dffab04b249f23850efd75b96e1_demo.mp4
I've used the audio of a 1Brown3Blue video. \
There is an already transcribed version, because it takes about 20 minutes to transcribe the whole video. \
The output is not formated yet, because I haven't decided on a specific AI model to use and they all have different output

## Building
### Frontend
- https://v2.tauri.app/start/prerequisites/
- `cargo install create-tauri-app --locked`
- run `cargo tauri dev` or `cargo tauri build` inside the notes-frontend directory

### Backend
- if you have a gpu get the required prerequesites from https://github.com/ggml-org/whisper.cpp?tab=readme-ov-file#nvidia-gpu-support
- if it's not nvidia change `cuda` in the Cargo.toml inside the notes-backend directory to your gpu
- see https://docs.rs/crate/whisper-rs/latest for gpu names
- if you don't have a gpu set `use_gpu` to false in the transcribe.rs file in the source directory inside the notes-backend directory
- install https://ollama.com/
- `ollama pull qwen2.5:14b`
- `cargo install --locked cargo-leptos`
- `rustup target add wasm32-unknown-unknown`
- run `cargo leptos watch` or `cargo build --release` inside the notes-backend direcory