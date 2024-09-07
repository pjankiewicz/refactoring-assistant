# Refactoring Assistant

`Refactoring Assistant` is a command-line tool that allows you to refactor code in multiple files based on instructions using OpenAI's GPT API. You can provide instructions either directly as a string or from a file, and the tool will apply changes to files matching a specific pattern (e.g., `*.py` for Python files).

## Features

- Uses OpenAI's GPT model to refactor code.
- Applies changes to multiple files matching a specific pattern.
- Allows instructions to be provided as a string or from a file.
- Can process any file type based on a given pattern.

## Requirements

- Rust toolchain installed on your machine.
- An OpenAI API key (stored in the `OPENAI_API_KEY` environment variable).

## Installation

1. Clone the repository or download the source code.
2. Navigate to the directory containing the source code.
3. Run the following command to install the tool as a CLI application:

   ```bash
   cargo install --path .
   ```

This will install `Refactoring Assistant` globally on your system.

## Usage

Once installed, you can run the tool using the following command:

```bash
refactoring-assistant -i <INSTRUCTION> -p <FILE_PATTERN> [-m <MODEL>]
```

### Command-line Arguments

- `-i, --instruction <INSTRUCTION>`: The instruction to follow or a path to a file containing instructions.
- `-p, --pattern <FILE_PATTERN>`: The file pattern to apply the changes (e.g., `*.py` for Python files).
- `-m, --model <MODEL>`: (Optional) The OpenAI model to use for the transformation. Defaults to `gpt-4`.

### Example

1. To refactor Python files (`*.py`) in the current directory, replacing all variable names that start with `old_` to start with `new_`, you can run:

   ```bash
   refactoring-assistant -i "Replace all variable names that start with 'old_' to 'new_'" -p "*.py"
   ```

2. Alternatively, if your instructions are in a file (`instructions.txt`), you can use:

   ```bash
   refactoring-assistant -i instructions.txt -p "*.rs"
   ```

3. If you'd like to specify a model other than the default (e.g., `gpt-4-turbo`), you can do:

   ```bash
   refactoring-assistant -i instructions.txt -p "*.js" -m "gpt-4-turbo"
   ```

## Environment Variables

- `OPENAI_API_KEY`: Your OpenAI API key must be set as an environment variable. You can set it using the following command:

  ```bash
  export OPENAI_API_KEY="your_api_key_here"
  ```

## Error Handling

- If a file can't be processed (due to API issues or file system errors), an error message will be printed for that file, and the tool will continue with the next file.

## License

This project is licensed under the MIT License.
