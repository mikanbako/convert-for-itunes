# Scripts to convert music files for iTunes

My scripts to convert from music files to MP3 files for iTunes.

* convert_for_itunes.py: A script that converts Ogg Vorbis or MP3 files in an album to MP3 files for iTunes.
* move_music_files.py: A script to move Ogg Vorbis or MP3 files to a structured directory and rename its files by its album name, artist, title and others.

## Requirements

The scripts required the following softwares.

* [Python](https://www.python.org/) 3.7 or above
* [liboggvorbis](https://github.com/AO-Yumi/vorbis_aotuv)
* [vorbis-tools](https://github.com/xiph/vorbis-tools)
* [vorbisgain](https://sjeng.org/vorbisgain.html)
* [lame](https://sourceforge.net/projects/lame/)
* [aacgain](http://aacgain.altosdesign.com/)

### Installing required Python packages

```bash
$ pip3 install --user -r requirements.txt
```

## Usage

Run scripts with --help option.
