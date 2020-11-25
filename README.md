# Randy
Conky inspired system info viewer written in Rust / GTK3

## Cheezeburgerz!
Conky was named after the puppet... so this thing is Randy
> A man’s gotta eat, Julian.

# Randy
Conky inspired system info viewer written in Rust / GTK3

## Features
* Configurable modules
  * Module list:
    * system - system info
    * cpus - all cpus usage stats bar
    * cpu_consumers - top N pids using cpu and their usage
    * mem_consumers - top N pids using mem and their usage
    * filesystem - usage of a given mounted filesystem
  * Can order the modules how you wish
  * Can enable/disable modules and sub items
* UI settings
  * color_bar - color of the usage **bars**
  * color_label - color of the "labels"
  * color_text - color of all other text
  * opacity - the base opacity of the randy window
  * decoration - hide/show window decorations
  * font_size
  * mod_top - modulo used to skip frames for getting top data
  * mod_fs - modulo used to skip frames for getting filesystem data
  * resizable
  * skip_taskbar - incase you want to see a Randy item in the taskbar
  * timeout - time in seconds to wait between frame updates
  * xpos - starting position x
  * ypos - starting position y

## Screenshot
![screenshot](https://raw.githubusercontent.com/iphands/ronky/main/assets/screenshot.png)
