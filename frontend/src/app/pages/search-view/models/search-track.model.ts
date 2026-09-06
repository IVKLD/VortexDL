import { PlayableTrack } from '@shared/models/music-track.model';

export interface SearchTrackItemRdo extends PlayableTrack {
    duration: number | null;
    playbackCount: number | null;
    genre: string | null;
}

export interface SearchRdo {
    tracks: SearchTrackItemRdo[];
    hasMore: boolean;
}

export interface DownloadDto {
    url: string;
}
