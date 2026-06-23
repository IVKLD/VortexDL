export enum AudioFormat {
    MP3 = 'mp3',
    FLAC = 'flac',
    WAV = 'wav',
    UNKNOWN = 'unknown',
}

export interface MusicTrack {
    id: number;
    artist: string;
    title: string;
    format: AudioFormat;
    artworkUrl: string | null;
    sourceUrl: string | null;
    createdAt: number;
    size: number;
    position: number;
}

export type MusicTracks = MusicTrack[];

export interface UiMusicTrack {
    data: MusicTrack;
    isActive: boolean;
    isPlaying: boolean;
    isSelected: boolean;
}

export enum MusicCardAction {
    PLAY = 'play',
    DELETE = 'delete',
    CLICK = 'click',
    TOGGLE_SELECT = 'toggleSelect',
}