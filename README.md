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
remove_old_pkgs [pkgs_directory]
```

If no `pkgs_directory` is provided, the program will looking into the current directory.
`pkgs_directory` will most likely `/var/cache/pacman/pkg` or a copy of it.

**WARNING** : you might need to run this command as root if you run it directly in `/var/cache/pacman/pkg`.

## Exit codes

- `1` : argument is not a directory
- `2` : input-output error (not read or write right on the directory for instance)
