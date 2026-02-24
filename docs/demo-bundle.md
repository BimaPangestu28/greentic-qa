# Demo Bundle Guide

This guide explains how to create and run a demo bundle with greentic-operator for testing messaging providers.

## Overview

A demo bundle is a self-contained directory that includes provider packs, configuration, and setup inputs for running local demos with greentic-operator.

## Prerequisites

- [greentic-operator](https://github.com/greentic-ai-org/greentic-operator) installed
- Provider pack(s) (e.g., `messaging-telegram.gtpack`)
- Public URL for webhooks (ngrok or cloudflare tunnel)

## Bundle Structure

```
demo-bundle/
├── greentic.demo.yaml           # Main config
├── setup-<provider>.yaml        # Provider setup input
├── providers/
│   ├── messaging/               # Messaging provider packs
│   │   └── messaging-telegram.gtpack
│   ├── events/                  # Event provider packs
│   └── secrets/                 # Secret provider packs
├── logs/                        # Log output
├── state/                       # Runtime state
└── .greentic/
    └── dev/
        └── .dev.secrets.env     # Dev secrets (optional)
```

## Creating a Demo Bundle

### 1. Scaffold a New Bundle

```bash
greentic-operator demo new my-demo-bundle
cd my-demo-bundle
```

### 2. Add Provider Packs

Copy your provider pack to the appropriate domain directory:

```bash
# For messaging providers
cp path/to/messaging-telegram.gtpack providers/messaging/

# For event providers
cp path/to/events-webhook.gtpack providers/events/
```

### 3. Create Setup Input

Create a YAML file with provider configuration:

```yaml
# setup-telegram.yaml
messaging:
  messaging-telegram:
    public_base_url: "https://your-public-url.ngrok.io"
    bot_token: "YOUR_BOT_TOKEN"
```

**Important:** The structure must be `domain > provider-id > config-fields`.

## Running the Demo

### Basic Start

```bash
greentic-operator demo start --bundle . --setup-input setup-telegram.yaml --cloudflared off --env dev --tenant dev
```

### With Cloudflare Tunnel (Auto)

```bash
greentic-operator demo start --bundle . --setup-input setup-telegram.yaml --env dev --tenant dev
```

### Command Options

| Option | Description |
|--------|-------------|
| `--bundle <PATH>` | Path to the bundle directory |
| `--setup-input <FILE>` | YAML/JSON file with provider setup inputs |
| `--cloudflared off` | Disable cloudflared tunnel |
| `--env dev` | Environment for secrets (dev/test) |
| `--tenant dev` | Tenant identifier |
| `--skip-setup` | Skip provider setup flows |
| `--verbose` | Enable verbose logging |

## Useful Commands

### List Packs
```bash
greentic-operator demo list-packs --bundle .
```

### List Flows
```bash
greentic-operator demo list-flows --bundle . --pack messaging-telegram
```

### Check Status
```bash
greentic-operator demo status --bundle .
```

### View Logs
```bash
greentic-operator demo logs --bundle .
```

### Send Test Message
```bash
greentic-operator demo send --bundle . --provider messaging-telegram --message "Hello"
```

### Run Diagnostics
```bash
greentic-operator demo run --bundle . --pack messaging-telegram --flow diagnostics
```

## Provider-Specific Setup

### Telegram

1. Create bot via [@BotFather](https://t.me/botfather)
2. Get bot token
3. Setup public URL (ngrok/cloudflare)

```yaml
messaging:
  messaging-telegram:
    public_base_url: "https://xxxx.ngrok-free.app"
    bot_token: "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
```

### Slack

1. Create Slack App at [api.slack.com](https://api.slack.com/apps)
2. Get Bot Token and Signing Secret
3. Configure OAuth scopes

```yaml
messaging:
  messaging-slack:
    public_base_url: "https://xxxx.ngrok-free.app"
    bot_token: "xoxb-..."
    signing_secret: "..."
```

### Microsoft Teams

1. Register app in Azure AD
2. Create Bot Channel Registration
3. Get App ID and Password

```yaml
messaging:
  messaging-teams:
    public_base_url: "https://xxxx.ngrok-free.app"
    app_id: "..."
    app_password: "..."
```

### WhatsApp (Cloud API)

1. Setup Meta Business Account
2. Create WhatsApp Business App
3. Get Access Token and Phone Number ID

```yaml
messaging:
  messaging-whatsapp:
    public_base_url: "https://xxxx.ngrok-free.app"
    access_token: "..."
    phone_number_id: "..."
    verify_token: "..."
```

## Troubleshooting

### Error: "env secrets backend is disabled for env 'demo'"

Use `--env dev --tenant dev`:
```bash
greentic-operator demo start --bundle . --setup-input setup.yaml --env dev --tenant dev
```

### Error: "unknown domain 'messaging-telegram'"

Check YAML structure - domain must be top-level:
```yaml
messaging:              # <- domain (messaging/events/secrets)
  messaging-telegram:   # <- provider ID
    config_field: "..."
```

### Error: "No .gtpack files found"

Ensure packs are in correct location:
```bash
# Correct
providers/messaging/messaging-telegram.gtpack

# Wrong
packs/messaging-telegram.gtpack
```

### Provider not detected

Run list-packs to verify detection:
```bash
greentic-operator demo list-packs --bundle .
```

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│  External   │────▶│   Webhook   │────▶│    Operator     │
│  Platform   │     │  (tunnel)   │     │   (Gateway)     │
└─────────────┘     └─────────────┘     └────────┬────────┘
                                                 │
                                                 ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│  External   │◀────│   Egress    │◀────│  Flow Runner    │
│    API      │     │  Service    │     │   (WASM)        │
└─────────────┘     └─────────────┘     └─────────────────┘
```

## Available Flows

Each messaging provider pack typically includes:

| Flow | Description |
|------|-------------|
| `setup_default` | Automatic setup with minimal config |
| `setup_custom` | Setup with advanced options |
| `update` | Update existing configuration |
| `remove` | Remove provider configuration |
| `diagnostics` | Health check for connections |
| `verify_webhooks` | Verify webhook is configured |
| `requirements` | Check requirements are met |

## Environment Variables

```bash
export GREENTIC_LOG_LEVEL=debug                # Verbose logging
export GREENTIC_DEV_SECRETS_PATH=./secrets.env # Custom secrets path
```
