#!/usr/bin/env python3

# Converts MP3, Flac or Ogg Vorbis files in an album to MP3 files for iTunes.

from mutagen.id3 import (
    TIT2, TALB, TSRC, COMM, TCON, TDRC, TRCK, TPOS, TPE1, TPE2, TXXX)

import argparse
import concurrent.futures
import logging
import os
import os.path
import shutil
import subprocess
import sys
import tempfile

import mutagen
import mutagen.id3

OGG_VORBIS_MIME_TYPE = 'audio/vorbis'
FLAC_MIME_TYPE = 'audio/flac'
MP3_MIME_TYPE = 'audio/mp3'

SUPPORTED_MIME_TYPES = (OGG_VORBIS_MIME_TYPE, FLAC_MIME_TYPE, MP3_MIME_TYPE)

EXCLUDING_FILE_EXTENSIONS = ('.log', '.pdf', '.txt', '.jpg', '.png')


class ConversionFailedException(Exception):
    def __init__(self, cause):
        self.cause = cause


def exists_all_files(all_files):
    for path in all_files:
        if not os.path.isfile(path):
            return False

    return True


def get_mp3_file_path(ogg_file_path, output_directory):
    basename = os.path.basename(ogg_file_path)
    filename = os.path.splitext(basename)[0]

    return os.path.join(output_directory, filename + '.mp3')


def get_mime_type(source_file):
    file_type = mutagen.File(source_file)

    if not file_type:
        return None

    for mime_type in SUPPORTED_MIME_TYPES:
        if mime_type in file_type.mime:
            return mime_type

    return None


def get_all_mime_type(all_source_files):
    assert(all_source_files)

    is_first = True
    mime_type = None

    for source_file in all_source_files:
        current_mime_type = get_mime_type(source_file)

        if not current_mime_type:
            raise ConversionFailedException(
                'The format of {} is unsupported.'.format(source_file))

        if is_first:
            mime_type = current_mime_type
            is_first = False
        elif mime_type != current_mime_type:
            raise ConversionFailedException(
                'The format of {} is different.'.format(source_file))

    return mime_type


def calculate_ogg_vorbis_gain(all_source_files):
    vorbisgain = shutil.which('vorbisgain')

    try:
        subprocess.run([vorbisgain, '-a'] + all_source_files, check=True)
    except subprocess.CalledProcessError as e:
        raise ConversionFailedException('Calculating gain is failed.')


def calculate_flac_gain(all_source_files):
    metaflac = shutil.which('metaflac')

    try:
        subprocess.run(
            [metaflac, '--add-replay-gain'] + all_source_files, check=True)
    except subprocess.CalledProcessError as e:
        raise ConversionFailedException('Calculating gain is failed.')


def calculate_mp3_gain(all_source_files):
    aacgain = shutil.which('aacgain')

    try:
        subprocess.run([aacgain, '-r', '-a'] + all_source_files, check=True)
    except subprocess.CalledProcessError as e:
        raise ConversionFailedException('Calculating gain is failed.')


MIME_TYPE_TO_GAIN_CALCULATION_FUNCTION = {
    OGG_VORBIS_MIME_TYPE: calculate_ogg_vorbis_gain,
    FLAC_MIME_TYPE: calculate_flac_gain,
    MP3_MIME_TYPE: calculate_mp3_gain,
}


def calculate_gain(all_source_files):
    mime_type = get_all_mime_type(all_source_files)

    if mime_type in MIME_TYPE_TO_GAIN_CALCULATION_FUNCTION:
        MIME_TYPE_TO_GAIN_CALCULATION_FUNCTION[mime_type](all_source_files)
    else:
        raise ConversionFailedException('The files are an unsupported format.')


def copy_tags(source_file_path, mp3_file_path):
    file_type = mutagen.File(source_file_path)
    id3 = mutagen.id3.ID3()

    utf8 = mutagen.id3.Encoding.UTF8
    latin1 = mutagen.id3.Encoding.LATIN1

    if 'title' in file_type:
        id3.add(TIT2(encoding=utf8, text=file_type['title']))
    if 'album' in file_type:
        id3.add(TALB(encoding=utf8, text=file_type['album']))

    if 'artist' in file_type:
        id3.add(TPE1(encoding=utf8, text=file_type['artist']))
    if 'album artist' in file_type:
        id3.add(TPE2(encoding=utf8, text=file_type['album artist']))
    if 'albumartist' in file_type:
        id3.add(TPE2(encoding=utf8, text=file_type['albumartist']))

    if 'genre' in file_type:
        id3.add(TCON(encoding=utf8, text=file_type['genre']))
    if 'date' in file_type:
        id3.add(TDRC(encoding=latin1, text=file_type['date']))

    if 'tracknumber' in file_type:
        id3.add(TRCK(encoding=latin1, text=file_type['tracknumber']))
    if 'track' in file_type:
        id3.add(TXXX(encoding=utf8, desc='TRACK', text=file_type['track']))
    if 'tracknum' in file_type:
        id3.add(
            TXXX(encoding=utf8, desc='TRACKNUM', text=file_type['tracknum']))
    if 'tracktotal' in file_type:
        id3.add(
            TXXX(
                encoding=utf8,
                desc='TRACKTOTAL',
                text=file_type['tracktotal']))

    if 'discnumber' in file_type:
        id3.add(TPOS(encoding=latin1, text=file_type['discnumber']))
    if 'disctotal' in file_type:
        id3.add(
            TXXX(
                encoding=utf8,
                desc='DISCTOTAL',
                text=file_type['disctotal']))

    if 'isrc' in file_type:
        id3.add(TSRC(encoding=utf8, text=file_type['isrc']))

    if 'comment' in file_type:
        id3.add(COMM(encoding=utf8, lang='eng', text=file_type['comment']))
    elif 'description' in file_type:
        id3.add(COMM(encoding=utf8, lang='eng', text=file_type['description']))

    if 'description' in file_type:
        id3.add(
           TXXX(
               encoding=utf8,
               desc='DESCRIPTION',
               text=file_type['description']))
    if 'itunes_cddb_1' in file_type:
        id3.add(
            TXXX(
                encoding=utf8,
                desc='ITUNES_CDDB_1',
                text=file_type['itunes_cddb_1']))

    id3.save(mp3_file_path)


def convert_wave_to_mp3(wave_file_path, mp3_file_path):
    lame = shutil.which('lame')

    try:
        subprocess.run(
            [lame,
             '-V5',
             '--silent',
             wave_file_path,
             mp3_file_path],
            check=True)
    except subprocess.CalledProcessError:
        raise ConversionFailedException(
            'Conversion from wave to MP3 is failed.')


def convert_ogg_vorbis_to_mp3(ogg_file_path, mp3_file_path):
    wave_file = tempfile.NamedTemporaryFile()
    wave_file.close()
    wave_file_path = wave_file.name

    ogg123 = shutil.which('ogg123')
    try:
        subprocess.run(
            [ogg123, '-q', '-d', 'wav', '-f', wave_file_path, ogg_file_path],
            check=True)
    except subprocess.CalledProcessError:
        os.remove(wave_file_path)
        raise ConversionFailedException(
            'Conversion from Ogg Vorbis to wave is failed.')

    try:
        convert_wave_to_mp3(wave_file_path, mp3_file_path)
    finally:
        os.remove(wave_file_path)

    copy_tags(ogg_file_path, mp3_file_path)


def convert_flac_to_mp3(flac_file_path, mp3_file_path):
    wave_file = tempfile.NamedTemporaryFile()
    wave_file.close()
    wave_file_path = wave_file.name

    flac = shutil.which('flac')
    try:
        subprocess.run(
            [flac,
             '-s',
             '-d',
             '--apply-replaygain-which-is-not-lossless',
             '-o',
             wave_file_path,
             flac_file_path],
            check=True)
    except subprocess.CalledProcessError:
        os.remove(wave_file_path)
        raise ConversionFailedException(
            'Conversion from Flac to wave is failed.')

    try:
        convert_wave_to_mp3(wave_file_path, mp3_file_path)
    finally:
        os.remove(wave_file_path)

    copy_tags(flac_file_path, mp3_file_path)


def convert_mp3_to_mp3(mp3_source_file_path, mp3_destination_path):
    lame = shutil.which('lame')
    try:
        subprocess.run(
            [lame,
             '-V5',
             '--silent',
             mp3_source_file_path,
             mp3_destination_path],
            check=True)
    except subprocess.CalledProcessError:
        raise ConversionFailedException(
            'Conversion from wave to MP3 is failed.')


MIME_TYPE_TO_CONVERSION_FUNCTION = {
    OGG_VORBIS_MIME_TYPE: convert_ogg_vorbis_to_mp3,
    FLAC_MIME_TYPE: convert_flac_to_mp3,
    MP3_MIME_TYPE: convert_mp3_to_mp3,
}


def convert_to_mp3(source_file_path, mp3_file_path):
    mime_type = get_mime_type(source_file_path)

    if mime_type in MIME_TYPE_TO_CONVERSION_FUNCTION:
        MIME_TYPE_TO_CONVERSION_FUNCTION[mime_type](
            source_file_path, mp3_file_path)
    else:
        raise ConversionFailedException(
            '{} is an unsupported format.' % (source_file_path))


def get_max_workers():
    cpu_count = os.cpu_count()

    return cpu_count if cpu_count else 1


def run_converting_task(source_file, output_directory):
    mp3_file_path = get_mp3_file_path(source_file, output_directory)

    convert_to_mp3(source_file, mp3_file_path)


def convert_for_itunes(all_source_files, output_directory):
    os.makedirs(output_directory, exist_ok=True)

    logging.info('Gain calculation.')
    calculate_gain(all_source_files)

    logging.info('Convert to MP3 for iTunes.')
    with concurrent.futures.ThreadPoolExecutor(
            max_workers=get_max_workers()) as executor:
        future_to_source = {
            executor.submit(
                run_converting_task, source_file, output_directory):
            source_file
            for source_file in all_source_files}

        for future in concurrent.futures.as_completed(future_to_source):
            try:
                future.result()
                logging.info('%s is converted.', future_to_source[future])
            except ConversionFailedException as e:
                logging.error(
                    '%s cannot be converted: %s',
                    future_to_source[future],
                    e.cause)


def filter_paths(all_paths):
    return [path for path in all_paths
            if os.path.splitext(path)[1].lower()
            not in EXCLUDING_FILE_EXTENSIONS]


def main():
    logging.basicConfig(format='%(levelname)s:%(message)s', level=logging.INFO)

    parser = argparse.ArgumentParser(
        description='''
        Converts MP3, Flac or Ogg Vorbis files in an album to MP3 files.''')
    parser.add_argument(
        'source_files',
        metavar='SOURCE',
        nargs='+',
        help='MP3, Flac or Ogg Vorbis files in an album.')
    parser.add_argument(
        'output_directory',
        metavar='DIR',
        help='An output directory that contains MP3 files.')

    arguments = parser.parse_args()

    source_files = filter_paths(arguments.source_files)
    if len(source_files) == 0 or not exists_all_files(source_files):
        logging.error('The source files are not found.')

        sys.exit(1)

    output_directory = arguments.output_directory

    if os.path.isfile(output_directory):
        logging.error('The output directory is a file.')

        sys.exit(1)

    try:
        convert_for_itunes(source_files, output_directory)
    except ConversionFailedException as e:
        logging.error(e.cause)


if __name__ == '__main__':
    main()
