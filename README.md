## Backintime restore revert software [![Build Status](https://travis-ci.com/0xpr03/restore_revert.svg?branch=master)](https://travis-ci.com/0xpr03/restore_revert)
Allows to undo a restore if the .backup.<Date> files are still present.

This tool is for accidentally restoring a folder in Backintime with the backup option enabled.
In that case you end up with something like `myfile.txt` and `myfile.txt.backup.20180517`. The latter one being your original, pre undo file. The other one the restored version.
restore_revert just deletes the non .backup.<Date> version and renames the other on to it's original name. This is done for all pairs of such restored-original files in the specified folder (and sub folders).
## Usage
```
    restore_revert [FLAGS] --dir <PATH>

FLAGS:
        --follow-symlink    Follow symlinks. Warning: use with caution, experimental!
    -h, --help              Prints help information
    -r, --rename-symlink    rename symlinks
    -s, --simulate          simulate revert, do not change files
    -V, --version           Prints version information
    -v, --verbose           verbose output

OPTIONS:
    -d, --dir <PATH>    start directory for revert
```

## Example
Folder with files
```
a
a.backup.20180518
b
b.backup.20180518
c
c.backup.20180518
d
d.backup.20180518
```
`a` is the restored (old) version of file `a`.  
`a.backup.20180518` is the version present until the restore (newer).  
restore_revert will delete `a` and rename `a.backup.20180518` to `a`, reverting the restore of `a` from the BIT backup.
