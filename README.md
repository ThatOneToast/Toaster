# Welcome to Toaster!

Toaster is a utility to generate shell commands and or run shell commands on a schedule. 

Toaster uses toml as a config for generating the Schedules or Commands.
Toaster automatically generates a base config file in your $HOME/.toaster directory
This is the provided default config file.

```toml
[settings]
threads = 1
default_row_length = 4

# All scheduled commands should be provided under system
# An example of how a sysytem is structured
# [system.testing] # After the `.` is the name of your system.
# description = "testing will send Hola! every 30 seconds"
# shell = "zsh"
# stages = [
#     "echo \"Hola!\" | tee output.log",
#     """ # Multiline stages are supported
#     """ # Multiline stages are supported
# ]
# schedules = [
#     "00:00:00:00:30" # This will be ran every 30 seconds
# ]
[system]

# All commands should be provided under command
# An example of how a command is structured
# [command.testing]
# description = "testing"
# shell = "zsh"
# stages = [
#     "%[color:cyan,o:-s] clear && echo \"===== Dirs ===== \" ",
#     "%[color:cyan,o:l6;] ls",
#     "echo Hello World",
#     "%[color:green] echo ok123"
# ]
[command]
```

The `threads` field in settings is the amount of threads systems should use in their thread pool.
All scheduled commands will be ran on the thread pool, normal commands are ran on the main thread.

using `--reload` will reparse your config and update systems and commands.
using `--flush` will release all outputs in the queue and write to the file.
