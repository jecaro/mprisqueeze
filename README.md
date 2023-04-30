# mprisqueeze

`mprisqueeze` is a wrapper over [squeezelite]. It starts [squeezelite] in the 
background and exposes an [MPRIS] interface to control it with [MPRIS] clients 
such as [playerctl].

By default, `mprisqueeze` will try to discover the [LMS] server on the local 
network. To specify a host and a port:

```bash
$ mprisqueeze -H somehost -P 9000
```

The default command line for [squeezelite] is: `squeezelite -n SqueezeLite`. It 
starts [squeezelite] registering itself on [LMS] with the name `SqueezeLite`. 
To use another name, one can use:

```bash
$ mprisqueeze -p my-player
```

The command to start [squeezelite] can be changed with the last arguments, 
preceded by a `--`, for example:

```bash
$ mprisqueeze -- squeezelite -f ./squeezelite.log -n {}
```

Note that `mprisqueeze` must know the name [squeezelite] will use. Therefore 
the [squeezelite] command line must contains the string `{}`. It is replaced by 
the player name when starting the process.

[MPRIS]: https://specifications.freedesktop.org/mpris-spec/latest/
[squeezelite]: https://github.com/ralph-irving/squeezelite
[playerctl]: https://github.com/altdesktop/playerctl
[LMS]: https://github.com/Logitech/slimserver
