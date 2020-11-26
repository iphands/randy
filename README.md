# Randy
Conky inspired system info viewer written in Rust / GTK3

## Cheezeburgerz!
Conky was named after the puppet... so this thing is Randy
> A man’s gotta eat, Julian.

## Goals
### Learn a ton
I'm new to Rust (you can tell in the code!), having fun and learning a lot.

### Frick Ricky, stay off the CPU
Strive to do things in as little cycles as possible.
Not do things as fast as possible (hence to parallel scans of /proc/*).
```shell
ps -eo etimes,times,command | grep randy
```

### Linux only
At the moment Randy only really runs on Linux. Running on ther operating systems is not a goal.
*Note:* somone shared a screenshot of [Randy running on Windows](https://raw.githubusercontent.com/iphands/ronky/main/assets/winderz.png) with WSfL though :D.

#### Speed tests
* Check out the `bench` directory/sub-crate for some speed testing
* Build with `--features timings` to see details about how long things take in Randy

## Features

### Configurable modules
* Module list:
  * system - system info
  * cpus - all cpus usage stats bar
  * cpu_consumers - top N pids using cpu and their usage
  * mem_consumers - top N pids using mem and their usage
  * filesystem - usage of a given mounted filesystem
* Can order the modules how you wish
* Can enable/disable modules and sub items

### UI settings
* color_bar - base color of the usage bars
* color_bar_med - color of the usage bars > 50% < 80%
* color_bar_high - color of the usage bars > 80%
* color_label - color of the "labels"
* color_text - color of all other text
* decoration - hide/show window decorations
* font_family - the CSS-style font family string (font names with spaces must be wrapped in escaped quotes, eg `fo_family: "\"Terminus (TTF)\", \"Liberation Mono\", monospace"`)
* font_size
* mod_top - modulo used to skip frames for getting top data
* mod_fs - modulo used to skip frames for getting filesystem data
* resizable - bool to make the GUI resizable
* skip_taskbar - in case you want to see a Randy item in the taskbar
* timeout - time in seconds to wait between frame updates
* xpos - starting position x
* ypos - starting position y

## Building
### Optional deps / features
By default Randy turns on support for getting temp from NVIDIA cards via NVML shared objects and CPU temps via lm-sensors.

You can disable these by simply building with `--no-default-features`

### Required build packages
I have only barely looked into packaging but [jhjaggars](https://github.com/jhjaggars) put some helpful hints for Raspbian and Fedora in here:
* https://github.com/iphands/randy/issues/2
* https://github.com/iphands/randy/issues/1

### Example
```shell
cargo run --release --no-default-features  # build with lm-sensors and nvml disabled
```

## Running
Randy needs to be pointed at a config Yaml.
It will look for the `deatult.yml` in `$PWD/config`. Should work ootb if you launch from the root of the Git repo.

If are launching Randy from elsewhere... point it at the config Yaml file of your choosing.
Example:
```shell
randy /tmp/configs/my_cool_config.yml
```

## Screenshot
![screenshot](https://raw.githubusercontent.com/iphands/ronky/main/assets/screenshot.png)
