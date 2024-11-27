# trek

The universal package manager 🌳

A package manager that wraps around multiple data formats for compatibility.

This is mainly meant for use with `m(os)s`/`mossOS`, but cross-platform behavior can be achieved.

The official handlers for repositories will be made in this order: Debian, Arch, and finally, Moss. Other handlers will be implemented as needed, which will be listed as optional blocks in `TODO.md`.

## How do I use Trek?

Before we begin, here are what some of the parameters mean,

`HANDLER` must be the name of an implemented handler, like "Debian", "Arch", or "Moss". "All" is also an accepted option.

`REPOSITORIES` and `MIRRORS` should be either an alias or URL of a server. If these are not specified it will go through every mirror and repository in each handler.

`ARCH` is the target architecture of the package indexes. This will be assumed to be only the host architecture unless specified. You may specify multiple, but these will be put into separate files. `-a` should almost NEVER be specified, as using a different architecture can be dangerous if used without knowledge. Trek will always warn you thoroughly before installing something not of your host arch unless specified in the `trek.toml` file.

### Sync mirrors/repositories

Sync your mirrors/repositories by using `trek sync`. This creates/updates a `trek.db` file either specified in the `trek.toml` file, or at `/etc/trek/` on Linux and `/common/trek/` on Moss.

```sh
# -m/--mirror -r/--repo -a/--arch
trek sync <HANDLER> -m <MIRRORS> -r <REPOSITORIES> -a <ARCH>
```

Example:

```sh
# In this situation, 'main' is aliased to 'https://deb.debian.org/debian/dists/stable/' inside of the configuration file.
# We don't specify `-a` because it is assumed to be our host arch.
trek sync Debian -m "main" -r "stable"
```

## Install packages

TODO!

## Remove packages

TODO!

More to come!
