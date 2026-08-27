import { PlayableTrack } from '@shared/models/music-track.model';

export interface SearchTrackItemRdo extends PlayableTrack {
    duration: number | null;
    playbackCount: number | null;
    genre: string | null;
}

export interface SearchResponseRdo {
    tracks: SearchTrackItemRdo[];
    hasMore: boolean;
}

export interface StreamUrlResponseRdo {
    url: string;
}

export interface DownloadRequestDto {
    url: string;
}

export enum SearchProvider {
    YouTube = 'youtube',
    SoundCloud = 'soundcloud',
}

export enum SearchDurationFilter {
    Any = 'any',
    Short = 'short',
    Medium = 'medium',
    Long = 'long',
    Epic = 'epic',
}

export interface SearchSettingsModel {
    provider: SearchProvider;
    duration?: SearchDurationFilter;
}
