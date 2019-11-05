#!/usr/bin/env python3

# Renames and moves music files to the directory.

import argparse
import logging
import pathlib
import os
import os.path
import re
import shutil
import sys

import filename_sanitizer
import mutagen

OGG_VORBIS_MIME_TYPE = 'audio/vorbis'
MP3_MIME_TYPE = 'audio/mp3'

DISC_TRACK_NUMBER_PATTERN = re.compile(r'^(\d+)(?:/\d*$)?')

UNKNOWN_ALBUM = 'Unknown album'
UNKNOWN_ARTIST = 'Unknown atrist'

EXCLUDING_FILE_EXTENSIONS = ('.log', '.pdf', '.txt', '.jpg', '.png')


class FileLoadingError(Exception):
    def __init__(self, path):
        self.path = path


class FileCopyingError(Exception):
    def __init__(self, source_path, destination_path):
        self.source_path = source_path
        self.destination_path = destination_path


class MusicInfo:
    def __init__(self, file_type):
        self.__file_type = file_type

    @property
    def file_type(self):
        return self.__file_type

    def get_album_artist(self):
        raise NotImplementedError()

    def get_album(self):
        raise NotImplementedError()

    def get_disc_number(self):
        raise NotImplementedError()

    def get_track_number(self):
        raise NotImplementedError()

    def get_track_name(self):
        raise NotImplementedError()


class OggVorbisInfo(MusicInfo):
    def __init__(self, file_type):
        MusicInfo.__init__(self, file_type)

    def get_album_artist(self):
        if 'album artist' in self.file_type:
            return self.file_type['album artist'][0]
        elif 'artist' in self.file_type:
            return self.file_type['artist'][0]
        else:
            return None

    def get_album(self):
        return (self.file_type['album'][0] if 'album' in self.file_type
                else None)

    def get_disc_number(self):
        if 'discnumber' not in self.file_type:
            return None

        match = DISC_TRACK_NUMBER_PATTERN.match(
            self.file_type['discnumber'][0])

        return match.group(1) if match else None

    def get_track_number(self):
        if 'tracknumber' not in self.file_type:
            return None

        match = DISC_TRACK_NUMBER_PATTERN.match(
            self.file_type['tracknumber'][0])

        return match.group(1) if match else None

    def get_track_name(self):
        return (self.file_type['title'][0] if 'title' in self.file_type
                else None)


class Mp3Info(MusicInfo):
    def __init__(self, file_type):
        MusicInfo.__init__(self, file_type)

    def get_album_artist(self):
        if 'TPE2' in self.file_type:
            return self.file_type['TPE2'].text[0]
        elif 'TXXX:ALBUMARTIST' in self.file_type:
            return self.file_type['TXXX:ALBUMARTIST'].text[0]
        elif 'TPE1' in self.file_type:
            return self.file_type['TPE1'].text[0]
        else:
            return None

    def get_album(self):
        return (self.file_type['TALB'].text[0] if 'TALB' in self.file_type
                else None)

    def get_track_number(self):
        if 'TRCK' not in self.file_type:
            return None

        match = DISC_TRACK_NUMBER_PATTERN.match(self.file_type['TRCK'].text[0])

        return match.group(1) if match else None

    def get_disc_number(self):
        if 'TPOS' not in self.file_type:
            return None

        match = DISC_TRACK_NUMBER_PATTERN.match(self.file_type['TPOS'].text[0])

        return match.group(1) if match else None

    def get_track_name(self):
        return (self.file_type['TIT2'].text[0] if 'TIT2' in self.file_type
                else None)


MIME_TYPE_TO_MUSIC_INFO = {
    OGG_VORBIS_MIME_TYPE: OggVorbisInfo,
    MP3_MIME_TYPE: Mp3Info,
}


class MusicFile:
    def __init__(self, path):
        self.path = path

        file_type = mutagen.File(path)

        if not file_type:
            raise FileLoadingError('Unsupported file: {}'.format(path))

        mime_type = self.get_mime_type(file_type)

        if mime_type not in MIME_TYPE_TO_MUSIC_INFO:
            raise FileLoadingError('Unsupported format: {}'.format(file_type))

        self.info = MIME_TYPE_TO_MUSIC_INFO[mime_type](file_type)

    @classmethod
    def get_mime_type(cls, file_type):
        for mime_type in MIME_TYPE_TO_MUSIC_INFO:
            if mime_type in file_type.mime:
                return mime_type

        return None


def is_excluding_file(path):
    extension = os.path.splitext(path)[1].lower()

    return extension in EXCLUDING_FILE_EXTENSIONS


def load_files(music_paths):
    music_files = []

    for path in music_paths:
        if is_excluding_file(path):
            continue

        try:
            music_files.append(MusicFile(path))
        except mutagen.MutagenError:
            raise FileLoadingError(path)

    return music_files


def get_new_filename(music_file):
    disk_number = music_file.info.get_disc_number()
    track_number = music_file.info.get_track_number()
    track_name = music_file.info.get_track_name()
    extension = os.path.splitext(music_file.path)[1]

    filename = ''

    if disk_number and track_number:
        filename += '{}-{}. '.format(disk_number, track_number)
    elif disk_number:
        filename += '{}-x. '.format(disk_number)
    elif track_number:
        filename += '{}. '.format(track_number)

    if track_name:
        filename += '{}{}'.format(track_name, extension)
    else:
        filename += music_file.path

    return filename_sanitizer.sanitize_path_fragment(filename)


def get_output_path(music_file, output_directory):
    album_artist = music_file.info.get_album_artist()
    album = music_file.info.get_album()

    album_artist = album_artist if album_artist is not None else UNKNOWN_ARTIST
    album = album if album is not None else UNKNOWN_ALBUM

    output_path = (
        pathlib.Path(output_directory) /
        filename_sanitizer.sanitize_path_fragment(album_artist) /
        filename_sanitizer.sanitize_path_fragment(album) /
        get_new_filename(music_file))

    return str(output_path)


def copy_music_file(music_file, is_dry_run_mode, output_directory):
    new_path = get_output_path(music_file, output_directory)

    logging.info('"%s" is moved to "%s"', music_file.path, new_path)

    if is_dry_run_mode:
        return

    if os.path.exists(new_path):
        raise FileCopyingError(music_file.path, new_path)

    os.makedirs(os.path.dirname(new_path), exist_ok=True)
    shutil.copy(music_file.path, new_path)


def move_music_files(all_music_files, is_dry_run_mode, output_directory):
    os.makedirs(output_directory, exist_ok=True)

    for music_file in all_music_files:
        copy_music_file(music_file, is_dry_run_mode, output_directory)

    if is_dry_run_mode:
        return

    for path in (music_file.path for music_file in all_music_files):
        os.remove(path)


def main():
    logging.basicConfig(format='%(levelname)s:%(message)s', level=logging.INFO)

    parser = argparse.ArgumentParser(
        description='Renames and moves music files to the directory.')
    parser.add_argument(
        '-d',
        '--dry-run',
        action='store_true',
        help='Run dry run mode. Files are not moved.')
    parser.add_argument(
        'musics', nargs='+', metavar='FILE', help='Music files.')
    parser.add_argument(
        'output_directory', metavar='DIRECTORY', help='The output directory.')

    arguments = parser.parse_args()

    if (os.path.exists(arguments.output_directory) and
            not os.path.isdir(arguments.output_directory)):
        logging.error('%s is not a directory.', arguments.output_directory)
        sys.exit(1)

    try:
        music_files = load_files(arguments.musics)
    except FileLoadingError as e:
        logging.error('%s cannot be loaded.', e.path)
        sys.exit(1)

    try:
        move_music_files(
            music_files, arguments.dry_run, arguments.output_directory)
    except FileCopyingError as e:
        logging.error(
            '%s cannot be moved. Because %s has already existed.',
            e.source_path,
            e.destination_path)


if __name__ == '__main__':
    main()
