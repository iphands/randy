# Randy on flatpak
## No access to /proc/$pids... dead end
While randy technically works in flatpak. The top/consumer modules do not work.

Flatpak sandboxes intentionally isolates the running process from other processes such that they can only see systemd and self.

Until I can resolve this (sorry not an expert here) I can't publish Randy Bobandy :(

### Edit
Looks like the official word is flatpak cant support those kinds of apps :(
https://github.com/flatpak/flatpak/issues/4001
