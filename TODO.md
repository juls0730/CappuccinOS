## TODO
not really meant to be looked at, just writing down my thoughts

- [ ] finish fat fs
    - [ ] follow cluster chains
    - [ ] Search for file in code
        - [ ] search into folders
    - [ ] Integrate with VFS
- [ ] Custom FS

2 partitions, one that is the FAT fs that the system boot from, the directory structure looks like this:
| Path         | FS type           |
|--------------|-------------------|
| /            | Custom            | 
| /bin         | Custom            | 
| /boot        | FAT32 (symlink)   |
| /boot/EFI    | FAT32             |
| /boot/limine | FAT32             |
