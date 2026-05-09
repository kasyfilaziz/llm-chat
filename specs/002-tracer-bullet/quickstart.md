# Quickstart Guide: Phase 1 Tracer Bullet

## Prerequisites
- Rust 1.80.0+ installed
- Dioxus CLI (`cargo install dioxus-cli --version 0.7.0`)
- Valid OpenAI API Key OR a local Ollama instance running.

## Configuration

1. Create a `.env` file in the root of the project:

```env
# Example for OpenAI
API_PROVIDER_TYPE=openai
API_ENDPOINT_URL=https://api.openai.com/v1/chat/completions
API_MODEL_NAME=gpt-4o-mini
API_KEY=sk-your-openai-key-here
```

```env
# Example for Ollama
API_PROVIDER_TYPE=ollama
API_ENDPOINT_URL=http://localhost:11434/api/chat
API_MODEL_NAME=llama3
API_KEY=not-needed-for-ollama
```

## Running the Application

To run the desktop application with hot-reloading:

```bash
dx serve --platform desktop
```
