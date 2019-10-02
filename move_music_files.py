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

DISC_TRACK_NUMBER_PATTERN = re.compile(r'^(\d+)(?:/\d*$)?')

# TODO MP3


class MusicFile:
    def __init__(self, path):
        self.path = path
        self.file_type = mutagen.File(path)


class FileLoadingError(Exception):
    def __init__(self, path):
        self.path = path


class FileCopyingError(Exception):
    def __init__(self, source_path, destination_path):
        self.source_path = source_path
        self.destination_path = destination_path


def load_files(music_paths):
    music_files = []

    for path in music_paths:
        try:
            music_files.append(MusicFile(path))
        except mutagen.MutagenError:
            raise FileLoadingError(path)

    return music_files


def get_album_artist(file_type):
    if 'album artist' in file_type:
        return file_type['album artist'][0]
    elif 'artist' in file_type:
        return file_type['artist'][0]
    else:
        return 'Unknown artist'


def get_album(file_type):
    return file_type['album'][0] if 'album' in file_type else 'Unknown album'


def get_disc_track_number(file_type, field_name):
    match = DISC_TRACK_NUMBER_PATTERN.match(file_type[field_name][0])

    return match.group(1) if match else None


def get_track_name(file_type):
    return file_type['title'][0] if 'title' in file_type else 'No title'


def get_new_filename(music_file):
    disk_number = get_disc_track_number(music_file.file_type, 'discnumber')
    track_number = get_disc_track_number(music_file.file_type, 'tracknumber')
    track_name = get_track_name(music_file.file_type)
    extension = os.path.splitext(music_file.path)[1]

    filename = ''

    if disk_number and track_number:
        filename += '{}-{}. '.format(disk_number, track_number)
    elif disk_number:
        filename += '{}-x. '.format(disk_number)
    elif track_number:
        filename += '{}. '.format(track_name)

    filename += '{}{}'.format(track_name, extension)

    return filename_sanitizer.sanitize_path_fragment(filename)


def get_output_path(music_file, output_directory):
    album_artist = get_album_artist(music_file.file_type)
    album = get_album(music_file.file_type)

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
