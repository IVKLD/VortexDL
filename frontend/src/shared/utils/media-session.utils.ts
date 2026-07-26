import { PlayableTrack } from '@shared/models/music-track.model';

export function updateMediaSessionMetadata(track: PlayableTrack, album = 'VortexDL'): void {
    if (!('mediaSession' in navigator)) return;

    navigator.mediaSession.metadata = new MediaMetadata({
        title: track.title,
        artist: track.artist,
        album,
        artwork: track.artworkUrl ? [{ src: track.artworkUrl }] : [],
    });
}

export function setMediaSessionPlaybackState(playing: boolean): void {
    if ('mediaSession' in navigator) {
        navigator.mediaSession.playbackState = playing ? 'playing' : 'paused';
    }
}

export interface MediaSessionActionHandlers {
    onPlay?: () => void;
    onPause?: () => void;
    onPreviousTrack?: () => void;
    onNextTrack?: () => void;
}

export function setupMediaSessionHandlers(handlers: MediaSessionActionHandlers): void {
    if (!('mediaSession' in navigator)) return;

    if (handlers.onPlay) {
        navigator.mediaSession.setActionHandler('play', handlers.onPlay);
    }
    if (handlers.onPause) {
        navigator.mediaSession.setActionHandler('pause', handlers.onPause);
    }
    if (handlers.onPreviousTrack) {
        navigator.mediaSession.setActionHandler('previoustrack', handlers.onPreviousTrack);
    }
    if (handlers.onNextTrack) {
        navigator.mediaSession.setActionHandler('nexttrack', handlers.onNextTrack);
    }
}
