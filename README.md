# Loggit

Loggit is a lightweight, easy-to-use logging library for Rust. It provides ready-to-use logger macros that let you start logging with zero boilerplate. You simply import and use it; no additional setup is required for basic usage. However, if you need more control, you can customize the logging format, colors, and minimum logging level.

## Features

- **Zero Setup**: Just import the library and start logging.
- **Customizable**: Change log formats, colors, and logging levels.
- **Macros Provided**: Includes `trace!`, `debug!`, `info!`, `warn!`, and `error!`.
- **Flexible Formatting**: Use custom templates with placeholders like `{level}`, `{file}`, `{line}`, and `{message}`.

## Installation

Add LogGit to your Cargo.toml:

````toml
[dependencies]
loggit = "0.1.0"
````

or just write in the terminal:
```shell
cargo add loggit
```

## Usage

### Basic Logging

Simply import the logger macros and use it in your project:

````rust
use loggit::{trace, debug, info, warn, error};

fn main() {
    trace!("This is a trace message.");
    debug!("Debug message: variable value = {}", 42);
    info!("Informational message.");
    warn!("Warning: something might be off.");
    error!("Error occurred: {}", "example error");
}
````

### Customizing the Log Level

Set the minimum log level so that only messages at that level and above are printed:

````rust
use loggit::logger::set_log_level;
use loggit::Level;

fn main() {
    // Set log level to DEBUG; TRACE messages will be ignored.
    set_log_level(Level::DEBUG);

    debug!("This is a debug message.");
    trace!("This trace message will not be logged.");
}
````

### Customizing the Log Format

You can adjust the log format globally or per log level. Templates can include placeholders like `{level}`, `{file}`, `{line}`, and `{message}`. Colors can be configured by wrapping text with color tags.

**Global Format Customization**

````rust
use loggit::logger::set_global_formatting;

fn main() {
    // Set a global custom log format using color tags.
    set_global_formatting("<green>[{level}]<green> ({file}:{line}) - {message}".to_string());

    info!("This info message follows the new global format.");
    info!("The error message as well.");
}
````

**Level-Specific Format Customization**

````rust
use loggit::logger::set_level_formatting;
use loggit::Level;

fn main() {
    // Customize the ERROR log format specifically.
    set_level_formatting(
        Level::ERROR,
        "<red>[{level}]<red> <blue>({file}:{line})<blue> - <red>{message}<red>".to_string()
    );

    error!("This error message will follow the custom error format.");
}
````

### Enabling Colorized Output

Enable or disable colored output based on your preference:

````rust
use loggit::logger::set_colorized;

fn main() {
    // Enable colored output.
    set_colorized(true);
    
    info!("This info message will be colorized as specified in the format.");
}
````

### Customizing Terminal Output

Control whether messages are printed directly to the terminal:

````rust
use loggit::logger::set_print_to_terminal;

fn main() {
    // Disable terminal output (for example, if you want to log to a file instead).
    set_print_to_terminal(false);
    
    info!("This message will not be printed to the terminal.");
}
````

## Configuration

Internally, LogGit uses a simple configuration structure which holds:
- **Log Level**: One of TRACE, DEBUG, INFO, WARN, or ERROR.
- **Terminal Output**: A flag that determines if logs are printed to the terminal.
- **Colorization**: A flag to enable or disable colored output.
- **Custom Formats**: Individual formatters for each log level.

The default configuration already provides sensible defaults, so you can get started right away. Customization is available for those who need advanced logging setups.

## Contributing

Contributions and suggestions are welcome! Feel free to open issues or submit pull requests to help improve LogGit.

## License

LogGit is licensed under the MIT License.
