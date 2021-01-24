# Randy
Conky inspired system info viewer written in Rust / GTK3

## Cheezeburgerz!
Conky was named after the puppet... so this thing is Randy
> A man’s gotta eat, Julian.

## Goals
### Learn a ton
I'm new to Rust (you can tell in the code!), having fun and learning a lot.

I'd like to come up with a more standard "module" interface once I've developed more of the modules and refactor based on what I have learned.

### Feature parity with my old Conky config
**Done!** My old Conky setup looked more or less like [the Randy screenshot](https://github.com/iphands/randy#screenshots).

Took 200+ commits to get there, but its there and working!

### Frick Ricky, stay off the CPU
Strive to do things in as little cycles as possible.
Not do things as fast as possible (hence to parallel scans of /proc/*).
```shell
ps -eo etimes,times,command | grep randy
```

#### Speed tests
* Check out the `bench` directory/sub-crate for some speed testing
* Build with `--features timings` to see details about how long things take in Randy

### Linux only
At the moment Randy only really runs on Linux. Running on other operating systems is not a goal.

*Note:* someone shared a screenshot of [Randy running on Windows](https://raw.githubusercontent.com/iphands/ronky/main/assets/winderz.png) with WSfL though :D.

## Features

### Configurable modules
* Module list:
  * system - system info
  * cpus - all cpus usage stats bar
  * cpu_consumers - top N pids using cpu and their usage
  * mem_consumers - top N pids using mem and their usage
  * filesystem - usage of a given mounted filesystem
  * net - usage recv/trans for a given network interface
  * battery - charging/discharging percentage of /sys/*/power_supply's
* Can order the modules how you wish
* Can enable/disable modules and sub items

### UI settings
* bar_height - the height of the bars (default: 10px)
* base_opacity - the base opacity of the Randy window.  affects `window` and all sub-widgets. (default: 1.0)
* color_bar - base color of the usage bars
* color_bar_med - color of the usage bars > 50% < 80%
* color_bar_high - color of the usage bars > 80%
* color_borders - color of the GTK *borders* (frame borders, bar borders) defaults to same as *color_text*
* color_label - color of the "labels"
* color_text - color of all other text
* decoration - hide/show window decorations
* font_family - the CSS-style font family string (font names with spaces must be wrapped in escaped quotes, eg `fo_family: "\"Terminus (TTF)\", \"Liberation Mono\", monospace"`)
* font_size
* mod_bat -modulo used to skip frames for getting battery data (default: 2)
* mod_fs - modulo used to skip frames for getting filesystem data (default: 2)
* mod_top - modulo used to skip frames for getting top data (default: 2)
* resizable - bool to make the GUI resizable
* skip_taskbar - in case you want to see a Randy item in the taskbar
* timeout - time in seconds to wait between frame updates
* xpos - starting position x
* ypos - starting position y

## Building
### Optional deps / features
* `nvidia`: Enable NVIDIA card temp sensing via NVML .so
* `sensors`: Enable lm-sensors integration

Example:
```shell
cargo run --features sensors,nvidia
```

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
It will look for the `default.yml` in `$PWD/config`. Should work ootb if you launch from the root of the Git repo.

If you are launching Randy from elsewhere... point it at the config Yaml file of your choosing.
Example:
```shell
randy /tmp/configs/my_cool_config.yml
```

## Screenshots
<table>
 <tr>
  <td><img src="https://raw.githubusercontent.com/iphands/ronky/main/assets/screenshot.png" alt="screenshot"></td>
  <td><img src="https://raw.githubusercontent.com/iphands/ronky/main/assets/green.png" alt="screenshot_green"></td>
 </tr>
</table>

## FAQ
* Does randy work with Wayland?
  * Yes, check out the [proof](https://raw.githubusercontent.com/iphands/ronky/main/assets/wayland.png)
* Does randy work with X?
  * Yes
