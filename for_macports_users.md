# For macports users

This document describes about installing packages by macports.

Build some packages to install latest version.

## [liboggvorbis](https://github.com/AO-Yumi/vorbis_aotuv)

```shell
$ chmod 755 configure
$ chmod 755 install-sh
$ ./configure --prefix=/opt/local --with-ogg=/opt/local --with-ogg-libraries=/opt/local/lib --with-ogg-includes=/opt/local/include
$ make
$ sudo make install
```

## [vorbis-tools](https://github.com/xiph/vorbis-tools)

```shell
$ sudo port install libao
$ ./autogen.sh
$ ./configure --prefix=/opt/local
$ make
$ sudo make install
```

## [vorbisgain](https://sjeng.org/vorbisgain.html)

```shell
$ ./configure --prefix=/opt/local --with-ogg-prefix=/opt/local --with-vorbis-prefix=/opt/local
$ make
$ sudo make install
```

## [lame](https://sourceforge.net/projects/lame/)

Edit lame-3.100/include/libmp3lame.sym: Remove lame_init_old. See also http://kameya-z.way-nifty.com/blog/2018/01/lame-3100.html.

```bash
$ ./configure --prefix=/opt/local
$ make
$ sudo make install
```

## [aacgain](http://aacgain.altosdesign.com/)

Use the provided package by macports.

Because building on macOS is difficult. Applying some patches are required. And aacgain package in macports is reverted from 1.9 to 1.8. See also https://github.com/macports/macports-ports/blob/master/audio/aacgain/Portfile.

```shell
$ sudo port install aacgain
```
