# mprisqueeze

[![CI][status-png]][status]

`mprisqueeze` is a wrapper over [squeezelite]. It starts [squeezelite] in the 
background and exposes an [MPRIS] interface to control it with [MPRIS] clients 
such as [playerctl].

By default, `mprisqueeze` will try to discover the [LMS] server on the local 
network. To specify a host and a port:

```bash
$ mprisqueeze -H somehost -P 9000
```

The default command line for [squeezelite] is:

```
squeezelite -n {name} -s {server}
```

Before calling [squeezelite], `mprisqueeze` replaces:
- `{name}` by the name of the player, `Squeezelite` by default
- `{server}` by the LMS server IP, either automatically discovered either set 
  with the `-H` switch

It then starts [squeezelite] registering itself on [LMS] with the name 
`SqueezeLite`. To use another name, one can use:

```bash
$ mprisqueeze -p my-player
```

The command to start [squeezelite] can be changed with the last arguments, 
preceded by `--`, for example:

```bash
$ mprisqueeze -- squeezelite -f ./squeezelite.log -n {name} -s {server}
```

Note that when using a custom command, both parameters must be present on the 
command line: `{name}` and `{server}`.

[status]: https://github.com/jecaro/mprisqueeze/actions
[status-png]: https://github.com/jecaro/mprisqueeze/workflows/CI/badge.svg
[MPRIS]: https://specifications.freedesktop.org/mpris-spec/latest/
[squeezelite]: https://github.com/ralph-irving/squeezelite
[playerctl]: https://github.com/altdesktop/playerctl
[LMS]: https://github.com/Logitech/slimserver
