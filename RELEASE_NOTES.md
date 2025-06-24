# Release notes 

## Version 0.1.9
- Improved documentation

## Version 0.1.8
- Huge performance improvement: the logic of file storage and writing has been reworked. See the [PR](https://github.com/DobbiKov/loggit/pull/23) for more information.

## Version 0.1.7
- Fixed critical error when the result of the set functions were always overwritten by default ones or the ones from the log files.

## Version 0.1.6
- Added a new feature of loading config from `env` variables.
- If the file `loggit.(json/ini/env)` are found in the root directory the library will automatically load the config from one of those files.
- If the `env` variables are set, the library will automatically set the config from those set variables.

## Version 0.1.5
- Added a new feature of loading config from `json`, `ini` and `env` files using `load_config_from_file` function.

## Version 0.1.4
- Improved error handling for setters
- Added module as an option to the log messages parts

## Version 0.1.3
- String optimization has been done across all the library

## Version 0.1.2
- Fixed an error with std::fs::exists

## Version 0.1.1
In this version of the library the file feature has been developed, thus this version contains the next features:
- adding files where to save logs to
- specifying filenames format
- add rotations
- add compression

## Version 0.1.0
That's the first version of the **loggit** library that implements the next features:
- working logging out of the box without any setup
- colorizing log output
- change the format of log
- let's you change the log level
- let's you print date, time, file and line
