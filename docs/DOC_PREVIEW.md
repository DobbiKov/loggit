# API Documentation for Loggit

This document details all public functions and macros provided by the Loggit library. Use it as a reference when integrating or customizing your logging configuration.
If you're a developer and you're studying the codebase, please follow this link: [developer docs](./DOC_DEVELOPER.md)

---

## Module: `logger`

### `set_file(format: &str)`

Initializes file logging by configuring a file name and format.

- **Description:**  
  Configures Loggit to write logs to a file. The `format` string is used to generate the file name and must include a file extension. The format may include placeholders such as:
  - `{time}` – Current time.
  - `{date}` – Current date.
  - `{level}` - Current loggin level.
  - Other literal text.

- **Allowed values:**  
  - The format string **must** end with a text section containing a file extension (e.g. `.txt` or `.log`).  
  - Any forbidden characters such as `<`, `>`, `&`, or `%` will cause configuration to fail.  
  - *Examples:*  
    - `"app_{date}_{time}.txt"`  
    - `"{level}-log-on-{date}.log"`

---

### `set_compression(ctype: &str)`

Enables file compression for log archival.

- **Description:**  
  Sets the compression type for log files. After file logging is configured, you can enable compression to archive old logs.

- **Allowed values:**  
  - Accepts only a single allowed value: `"zip"`.  
  - Any other string will output an error and leave the compression configuration unchanged.

---

### `add_rotation(constraint: &str)`

Adds a new constraint for rotating log files.

- **Description:**  
  Adds a rotation strategy so that log files are rotated based on either time or file size. When a log file “expires” under the configured constraint, a new file is automatically created (and optionally compressed).

- **Allowed values:**  
  The `constraint` string can be in one of the following formats:
  - **Period rotation:**  
    - Numeric value followed by a unit:  
      - `"1 hour"`, `"2 day"`, `"33 week"`, `"6 month"`, `"12 year"`  
      - The unit is case sensitive and must match exactly (e.g. `" hour"`, `" day"`, etc.).
  - **Time-based rotation:**  
    - Time in a 24‑hour format using a colon separator:  
      - `"HH:MM"` (e.g. `"12:30"`).
  - **Size-based rotation:**  
    - Numeric value followed by a size unit:  
      - `"500 KB"`, `"5 MB"`, `"1 GB"`, or `"2 TB"`  
      - Note the space before the unit.

- If an incorrect value is provided, the rotation is not added and an error message is logged.

---

### `set_log_level(lvl: Level)`

Sets the minimum log level that will be processed.

- **Description:**  
  Changes the global logging threshold. Messages with a log level lower than the specified value are ignored.

- **Allowed values:**  
  The `lvl` parameter must be one of the variants from the public [`Level`](#level-enum) enum:
  - `Level::TRACE`
  - `Level::DEBUG`
  - `Level::INFO` (default)
  - `Level::WARN`
  - `Level::ERROR`

---

### `set_print_to_terminal(val: bool)`

Controls whether log messages are printed to the terminal.

- **Description:**  
  Enables or disables terminal output. When set to `false`, log messages are written only to the file (if configured) and are not printed on-screen.

- **Allowed values:**  
  - `true` – Enable terminal output.
  - `false` – Disable terminal output.

---

### `set_colorized(val: bool)`

Enables or disables colorized log output.

- **Description:**  
  When enabled, log messages printed to the terminal will include ANSI color codes based on the formatting configuration.

- **Allowed values:**  
  - `true` – Log messages are colorized.
  - `false` – Log messages are printed without color.

---

### `set_global_formatting(format: &str)`

Sets the log format globally for all log levels.

- **Description:**  
  Updates the formatter for every log level (TRACE, DEBUG, INFO, WARN, ERROR) to use the provided format string.

- **Allowed values:**  
  - Any valid formatting string that may include placeholders such as `{level}`, `{file}`, `{line}`, and `{message}`.  
  - You may also use color tags (e.g. `<red>`) which are interpreted by the formatter.
  
  *Example:*  
  - `"<green>[{level}]<green> ({file}:{line}) - {message}"`

---

### `set_level_formatting(level: Level, format: &str)`

Sets a custom log format for a specific log level.

- **Description:**  
  Updates the log formatting only for the specified level.

- **Allowed values:**  
  - **`level`:** Must be one of the following from the [`Level`](#level-enum) enum:
    - `Level::TRACE`
    - `Level::DEBUG`
    - `Level::INFO`
    - `Level::WARN`
    - `Level::ERROR`
  - **`format`:** A valid formatting string (as described above) that may include placeholders and color tags.
  
  *Example:*  
  - `set_level_formatting(Level::ERROR, "<red>[{level}]<red> (<blue>{file}:{line}<blue>) - <red>{message}<red>")`

---

### `init()`

Initializes the logger with the default configuration.

- **Description:**  
  Sets up Loggit with sensible defaults (e.g. level is `INFO`, terminal output enabled, default formatting per log level). This must be called before any logging macros are used if you wish to override the defaults. 
  
- **Allowed values:**  
  This function does not take parameters. It always initializes the logger to its default state.

---

## Macros

The following macros are publicly exported to provide an easy-to-use logging interface. They all use Rust format syntax and automatically capture the file name and line number.

### `trace!(...)`

- **Description:**  
  Logs a message at the **TRACE** level.  
- **Usage Example:**  
  ```rust
  trace!("Entering function {} with value {}", "my_func", 42);
  ```

---

### `debug!(...)`

- **Description:**  
  Logs a message at the **DEBUG** level.
- **Usage Example:**  
  ```rust
  debug!("Debug details: variable = {}", some_variable);
  ```

---

### `info!(...)`

- **Description:**  
  Logs a message at the **INFO** level.
- **Usage Example:**  
  ```rust
  info!("Application started successfully.");
  ```

---

### `warn!(...)`

- **Description:**  
  Logs a message at the **WARN** level.
- **Usage Example:**  
  ```rust
  warn!("Configuration file {} not found; using defaults.", config_file);
  ```

---

### `error!(...)`

- **Description:**  
  Logs a message at the **ERROR** level.
- **Usage Example:**  
  ```rust
  error!("An error occurred: {}", error_detail);
  ```

---

## Enum: `Level`

Represents the logging severity level.

- **Variants:**
  - `Level::TRACE` – Most detailed messages for tracing computations.
  - `Level::DEBUG` – Debugging information.
  - `Level::INFO` – General information (default level).
  - `Level::WARN` – Warning messages.
  - `Level::ERROR` – Error messages indicating a problem.

---

Use this API reference as a guide for integrating and configuring Loggit in your projects. Adjust the formatting and logging parameters according to the needs of your application and deployment environment.
