import { PlayableTrack } from '@shared/models/music-track.model';
import {
    setMediaSessionPlaybackState,
    setupMediaSessionHandlers,
    updateMediaSessionMetadata,
} from '@shared/utils/media-session.utils';

export class MediaSessionManagerService {
    public initHandlers(callbacks: {
        onPlay: () => void;
        onPause: () => void;
        onPreviousTrack: () => void;
        onNextTrack: () => void;
    }): void {
        setupMediaSessionHandlers(callbacks);
    }

    public updateMetadata(track: PlayableTrack | null): void {
        if (track) {
            updateMediaSessionMetadata(track);
        }
    }

    public updatePlaybackState(isPlaying: boolean): void {
        setMediaSessionPlaybackState(isPlaying);
    }
}
