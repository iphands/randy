# Randy
Conky inspired system info viewer written in Rust / GTK3

## Cheezeburgerz!
Conky was named after the puppet... so this thing is Randy
> A man’s gotta eat, Julian.

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
* resizable
* skip_taskbar - incase you want to see a Randy item in the taskbar
* timeout - time in seconds to wait between frame updates
* xpos - starting position x
* ypos - starting position y

## Building
### Optional deps / features
By default Randy turns on support for getting temp from NVidia cards via NVML shared objects and CPU temps via lm-sensors.

You can disable these by simply building with `--no-default-features`

### Required build packages
I have only barely looked into packaging but [jhjaggars](https://github.com/jhjaggars) put some helpful hints for Raspbian and Fedora in here:
* https://github.com/iphands/randy/issues/2
* https://github.com/iphands/randy/issues/1

### Example
```shell
cargo run --release --no-default-features  # build with lm-sensors and nvml disabled
```

## Screenshot
![screenshot](https://raw.githubusercontent.com/iphands/ronky/main/assets/screenshot.png)
