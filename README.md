# studiole-report

A concrete `Report<T>` type that implements `miette::Diagnostic` for rich error rendering.

## Highlights

- **Context-typed errors** - `Report<T>` is generic over your error enum, giving you typed access to the current context via `current_context()`
- **Source chain** - `change_context()` wraps one report as the source of another, building an error chain that traces the path from root cause to high-level failure
- **Structured attachments** - attach diagnostic data like file paths, retry counts, or request IDs without polluting error variants
- **Diagnostic rendering** - implements `miette::Diagnostic` for graphical output with error codes derived from the context type
- **Standard error integration** - implements `std::error::Error`, `Display`, and `Debug`, so it works with `?` and any code expecting `dyn Error`
- **Inspired by [`error-stack`](https://docs.rs/error-stack/latest/error_stack/)**. `Report<T>` implements `std::error::Error` so it composes naturally with the standard error ecosystem.

## Usage

```rust
use studiole_report::prelude::*;

fn read_config(path: &Path) -> Result<String, Report<ConfigError>> {
    std::fs::read_to_string(path)
        .change_context(ConfigError::Read)
        .attach_path(&path)
}

fn initialize() -> Result<Config, Report<InitError>> {
    let raw = read_config(Path::new("/etc/app.toml"))
        .change_context(InitError::Config)?;
    parse(raw)
}
```

Each layer calls `change_context()` to wrap the previous error, building a chain:

```text
InitError::Config
  Caused by: ConfigError::Read
  ▷ path: /etc/app.toml
    Caused by: No such file or directory (os error 2)
```

## Features

- **`render`** - enables `Report::render()` for graphical diagnostic output via `miette/fancy`

## License

This repository and its libraries are provided open source with the [AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html) license that requires you must disclose your source code when you distribute, publish, or provide access to modified or derivative software.

Developers who wish to keep modified or derivative software proprietary or closed source can [get in touch for a commercial license agreement](https://studiole.uk/contact/).

> Copyright © Laurence Elsdon 2025-2026
>
> This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
>
> This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
>
> You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

> [GNU Affero General Public License](LICENSE.md)
