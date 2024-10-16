# Rust Text Analyzer

Rust Text Analyzer is a powerful tool for analyzing texts using Rust. It offers various features to extract useful information from texts.

## Features

- Word counting
- Word frequency calculation
- N-gram generation (bigrams, trigrams, etc.)
- Average word length calculation
- Identification of longest sentences
- Punctuation statistics analysis
- Banned word filtering

## Installation

Ensure you have Rust installed on your system. Then, clone this repository:
```
git clone https://github.com/yu-Celik/rust-text-analyzer.git
cd rust-text-analyzer
```

## Usage

To use the text analyzer, run the following command:

```
cargo run -- <path_to_text_file> <path_to_banned_words_file>
```

Example :
```
cargo run -- programming_tips.txt stop_words_french.txt
```

## Project Structure

- `src/lib.rs`: Contains the main logic of the text analyzer.
- `src/main.rs`: Entry point of the application.
- `src/tests.rs`: Unit tests to verify the analyzer's functionality.

## Tests

To run the unit tests, use the command:
```
cargo test
```

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
