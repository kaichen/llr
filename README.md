# LLR

LLR, Large Local Router.

## Features
- Forward LLM API requests to specified endpoints
- Log LLM request and response content to log file(with --dump-boby option)
- Cast OpenAI <=> Anthropic format (TODO)
- Load balance to multiple endpoints (TODO)

## Installation

```bash
# Clone the repository
$ git clone https://github.com/kaichen/llr.git
$ cd llr

# Build the project (requires Rust toolchain)
$ cargo build --release
```

## Quick Start

```bash
# Run the server with default configuration
$ cargo run --release
```

## Configuration(TODO)

LLR can be configured via a configuration file or environment variables. Example configuration:

```toml
# config.toml
[server]
port = 8080

[logging]
log_file = "llr.log"

[llm]
endpoints = [
  "http://localhost:8001",
  "http://localhost:8002"
]
```

## Usage

Send your LLM API requests to the LLR server endpoint. The server will forward, log, and (optionally) balance requests according to your configuration.

## Contribution

Contributions are welcome! Please open issues or pull requests. For major changes, please discuss them first.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
