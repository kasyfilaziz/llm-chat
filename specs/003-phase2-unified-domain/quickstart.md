# Quickstart: Phase 2 Configuration

Lumina Phase 2 transitions from strict `.env` file reliance to a hybrid configuration where user settings are persisted in a YAML file for easy UI manipulation, while `.env` serves as a baseline fallback or development override.

## 1. Directory Structure

Upon first launch, Lumina will automatically generate a configuration directory depending on your OS (via the `directories` crate):
- **Linux:** `~/.config/lumina/settings.yaml`
- **macOS:** `~/Library/Application Support/dev.lumina.app/settings.yaml`
- **Windows:** `C:\Users\Username\AppData\Roaming\dev.lumina.app\settings.yaml`

## 2. Default `settings.yaml`
If no configuration file exists, the application generates a default based on your `.env` file from Phase 1, or writes the following skeleton:

```yaml
provider_preferences:
  default_llm: "openai"
  openai_api_key: "" 
  ollama_endpoint: "http://127.0.0.1:11434"
mcp_servers: []
```

## 3. Adding an MCP Server Locally

To test the MCP foundation, you must have an MCP server installed locally.
Example using Node.js to install the filesystem server:

```bash
# 1. Ensure you have npx installed (Node.js)
node -v
npx -v

# 2. Add the configuration to your settings.yaml manually (or via UI once built)
# Note: Provide the absolute path to npx if you encounter PATH issues.
```

```yaml
mcp_servers:
  - name: "Local Filesystem"
    path: "npx"
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/your/target/directory"]
```

## 4. Run the Application
Launch Lumina using the Dioxus CLI:
```bash
dx serve
```
If the MCP server is configured correctly, you should see logs indicating a successful `stdio` handshake with the `Local Filesystem` server.