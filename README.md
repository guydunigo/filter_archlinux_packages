# Remove old Archlinux packages

## Description

This crate looks at all the packages archives in a given directory and remove all that have a newer version present in the same directory.
For example if you have the files
`linux-5.3.1.arch1-1-x86_64.pkg.tar.xz` and `linux-5.3.arch1-1-x86_64.pkg.tar.xz`,
the latter will be removed.
Ambiguous versions that can't be compared and non-packages files (not finishing by `.pkg.tar.*` are listed and ignored.

## Pacman 6 and sig files

Apparently pacman 6 now downloads sig files along with the packages, support for them has been added :
1. If it corresponds (same path except the `.sig` extension) to an old package to be deleted, it will be deleted as well.
2. Otherwise, if it corresponds to a package we keep, then we keep it.
3. Finally, if it doesn't belong to either categories, then we ignore it.

## Usage

```shell
remove_old_pkgs [-hd0123] [pkgs_directory]
```

If no `pkgs_directory` is provided, the program will look into the current directory.
`pkgs_directory` will most likely `/var/cache/pacman/pkg` or a copy of it.

**WARNING** : you might need to run this command as root if you run it directly in `/var/cache/pacman/pkg`.

- `-h` : help message
- `-d` : dry run, doesn't delete anything or change any file
- `-0..3` : auto-confirm/interractivity levels, the higher the number, the more we ask
    - `-0` : Doesn't ask anything and selects the default version in case of ambiguities
    - `-1` : Ask only before removing anything
    - `-2` : Ask when there are ambiguities regarding versions and before removing anything.
    - `-3` : Ask for every decision for every version comparison (even if we can clearly determine the latest one by ourselves)

## Exit codes

- `1` : unknown command-line option (or directory name starting with '-')
- `2` : argument is not a directory
- `3` : input-output error (not read or write right on the directory for instance)
