export enum AudioFormat {
    MP3 = 'mp3',
    FLAC = 'flac',
    WAV = 'wav',
    UNKNOWN = 'unknown',
}

export interface PlayableTrack {
    id: number;
    artist: string;
    title: string;
    artworkUrl: string | null;
    sourceUrl: string | null;
}

export interface MusicTrack extends PlayableTrack {
    format: AudioFormat;
    createdAt: number;
    size: number;
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