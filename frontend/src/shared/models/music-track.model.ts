export enum AudioFormat {
    MP3 = 'mp3',
    FLAC = 'flac',
    WAV = 'wav',
    M4A = 'm4a',
    AAC = 'aac',
    OGG = 'ogg',
    OPUS = 'opus',
    WMA = 'wma',
    ALAC = 'alac',
    AIFF = 'aiff',
    UNKNOWN = 'unknown',
}

export interface PlayableTrack {
    id: number;
    artist: string;
    title: string;
    artworkUrl?: string;
    sourceUrl?: string;
    permalinkUrl?: string;
    streamUrl?: string;
}

export interface MusicTrack extends PlayableTrack {
    format: AudioFormat;
    createdAt: number;
    size: number;
}

export type MusicTracks = MusicTrack[];
