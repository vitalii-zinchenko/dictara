<div align="center">

<img src="src-tauri/icons/Square310x310Logo.png" alt="Dictara" width="128" height="128">

# Dictara

**Typing is slow. Speaking isn't.**

Free · Bring Your Own Key · Speech-to-Text

Turn your spoken words into text — in any app, any language.

[![Download](https://img.shields.io/badge/Download-Dictara-blue?style=for-the-badge)](https://dictara.app/)

[**Get Dictara**](https://dictara.app/)

</div>

---

## How It Works

1. **Install** — Download and install Dictara
2. **Configure** — Add your OpenAI or Azure OpenAI API key
3. **Dictate** — Hold `FN` to record, release to transcribe. Or press `FN+Space` for hands-free mode
4. **Done** — Text is automatically pasted wherever your cursor is — in any app

---

## Troubleshooting

### Emoji Picker Appears When Using Fn Key

If the emoji picker (or character viewer) appears when you press the Fn/Globe (🌐) key, you need to change your macOS keyboard settings:

**Via System Settings (Recommended):**
1. Open **System Settings** → **Keyboard**
2. Find **"Press 🌐 key to"** dropdown
3. Change it to **"Do Nothing"** or **"Change Input Source"**

**Via Terminal:**
```bash
# Set Globe key to "Do Nothing"
defaults write com.apple.HIToolbox AppleFnUsageType -int 0

# Then log out and log back in, or restart your Mac
```

> **Note:** This is a macOS limitation. The Fn/Globe key triggers the emoji picker at a system level that applications cannot intercept.

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](.github/CONTRIBUTING.md) to get started.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
