# poke-http

An interactive terminal client for `.http` files.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- 📄 Parse `.http` files (VS Code REST Client / IntelliJ format)
- 🖥️ Interactive TUI for browsing and executing requests
- 🎨 Syntax-highlighted methods and pretty-printed JSON responses
- ⚡ Fast and lightweight

## Installation

```bash
cargo install poke-http
```

Or build from source:

```bash
git clone https://github.com/mmodzelewski/poke-http
cd poke-http
cargo install --path .
```

## Usage

```bash
poke api.http
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Execute selected request |
| `Tab` | Switch focus between panels |
| `q` | Quit |
| `Ctrl+C` | Quit |

### .http File Format

```http
### Get all users
GET https://api.example.com/users
Authorization: Bearer token123

### Create a user
POST https://api.example.com/users
Content-Type: application/json

{
    "name": "John Doe",
    "email": "john@example.com"
}
```

- Requests are separated by `###`
- Comments start with `#` or `//`
- Headers follow the request line
- Body comes after a blank line

## Roadmap

- [x] Variable substitution (`{{baseUrl}}`)
- [ ] Environment files
- [ ] Request history
- [ ] Edit requests interactively
- [ ] Save/export responses
- [ ] Request chaining
