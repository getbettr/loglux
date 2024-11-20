Loglux
======

[![License](https://img.shields.io/badge/license-UNLICENSE-blue.svg?style=flat)](https://github.com/rarescosma/loglux/blob/master/UNLICENSE)

Loglux is a simple rust application with portable binaries to control brightness.

While heavily inspired by the original [lux][lux], it differs from it in one major aspect:

The brightness control is [logarithmic][weber-fechner] - as we approach darker and
darker brightness values the control step gets smaller and smaller.

It's perfect for us creatures of the night watching rust streams in complete
darkness at 1AM as it allows us to make the laptop screen *really* dark.

## Installation

Binaries for Linux on various architectures are available on the [releases][releases] page.

They are statically linked against [musl][musl] to completely reduce runtime dependencies.

> [!NOTE] 
> Your user _must_ have write rights for the `/sys/class/backlight/.../brightness`
> file you're planning to use.
>
> Your best bet is to do a `ls -a /sys/class/backlight/*/brightness` and check if
> it's owned by the `video` group and if the group has write rights.
>
> If that's the case, simply add your user to the `video` group: `sudo usermod -aG video $USER`
>
> For a more involved solution using `udev` you could first add your user to the `wheel` group:
> 
> ```
> sudo usermod -aG wheel $USER
> ```
>
> Then define the udev rules to ensure the brightness file is writeable:
>
> ```
> sudo tee /etc/udev/rules.d/99-loglux.rules <<EOF
> RUN+="/bin/chgrp wheel /sys/class/backlight/intel_backlight/brightness"
> RUN+="/bin/chmod g+w /sys/class/backlight/intel_backlight/brightness"
> EOF
> ```
>
> Finally, trigger udev: 
>
> ```
> sudo udevadm control --reload-rules && sudo udevadm trigger
> ```

## Usage

```
loglux OPERATION [-p|--path (default: /sys/class/backlight)] [-n|--num-steps (default: 75)]
```

* `OPERATION` is either `up` or `down`
* `--path` can be either a start directory containing multiple controllers, or a path to specific controller.
  In the directory case, the controller with the highest `max_brightness` setting will be selected.
* `--num-steps` is the only tunable parameter and it specifies the total number of steps for the
  adjustment scale. The default is tuned for steps of 9-10% near the maximum, then they'll get smaller
  and smaller as we approach the minimum.

[lux]: https://github.com/Ventto/lux

[weber-fechner]: https://en.wikipedia.org/wiki/Weber%E2%80%93Fechner_law

[releases]: https://github.com/rarescosma/loglux/releases

[musl]: https://musl.libc.org/
