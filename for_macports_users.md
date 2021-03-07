# For macports users

This document describes about installing packages by macports.

Build some packages to install latest version.

## [vorbisgain](https://sjeng.org/vorbisgain.html)

If you cannot install vorbisgain by ports. See the following.

1. Download latest vorbisgain.
2. Build and install.

```shell
$ ./configure --prefix=/opt/local --with-ogg-prefix=/opt/local --with-vorbis-prefix=/opt/local
$ make
$ sudo make install
```

When the errors are occured while building, edit mics.c.

1. Add "#include <unistd.h>"
2. Add "#include <sys/ioctl.h>"
