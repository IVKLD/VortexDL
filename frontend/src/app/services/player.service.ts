import { effect, inject, Injectable, NgZone, signal } from '@angular/core';
import { PlayableTrack } from '@shared/models/music-track.model';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { NotificationService } from './notification.service';
import { AudioEngineService } from './player/audio-engine.service';
import { PlayerQueueService } from './player/player-queue.service';
import { MediaSessionManagerService } from './player/media-session-manager.service';

@Injectable({
    providedIn: 'root'
})
export class PlayerService {
    private readonly _musicState = inject(MusicTracksViewState);
    private readonly _notification = inject(NotificationService);
    private readonly zone = inject(NgZone);

    private readonly audioEngine = new AudioEngineService();
    private readonly queueService = new PlayerQueueService();
    private readonly mediaSession = new MediaSessionManagerService();

    public readonly currentTrack = signal<PlayableTrack | null>(null);
    public readonly loadingTrackId = signal<number | null>(null);

    public readonly queue = this.queueService.queue;
    public readonly shuffle = this.queueService.shuffle;

    public readonly isPlaying = this.audioEngine.isPlaying;
    public readonly progress = this.audioEngine.progress;
    public readonly duration = this.audioEngine.duration;
    public readonly volume = this.audioEngine.volume;

    constructor() {
        this.setupAudioEngineCallbacks();
        this.setupMediaSession();
        this.setupKeyboardShortcuts();

        effect(() => {
            const track = this.currentTrack();
            this.mediaSession.updateMetadata(track);
        });

        effect(() => {
            const playing = this.isPlaying();
            this.mediaSession.updatePlaybackState(playing);
        });
    }

    private setupAudioEngineCallbacks(): void {
        this.audioEngine.onEnded = () => this.next();
        this.audioEngine.onError = () => {
            const current = this.currentTrack();
            if (current) {
                this.handlePlaybackError(current, 'Audio playback failed');
            }
        };
    }

    private setupMediaSession(): void {
        this.mediaSession.initHandlers({
            onPlay: () => this.togglePlay(),
            onPause: () => this.togglePlay(),
            onPreviousTrack: () => this.previous(),
            onNextTrack: () => this.next(),
        });
    }

    private setupKeyboardShortcuts(): void {
        this.zone.runOutsideAngular(() => {
            window.addEventListener('keydown', (event: KeyboardEvent) => {
                if (event.code === 'Space') {
                    const target = event.target as HTMLElement;
                    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) {
                        return;
                    }
                    if (this.currentTrack()) {
                        event.preventDefault();
                        this.zone.run(() => {
                            this.togglePlay();
                        });
                    }
                }
            });
        });
    }

    private handlePlaybackError(track: PlayableTrack, message: string): void {
        if (this.currentTrack()?.id === track.id) {
            this.audioEngine.stop();
            this.currentTrack.set(null);
            this.loadingTrackId.set(null);
            this._notification.error(message);
        }
    }

    private playSource(track: PlayableTrack, src: string): void {
        if (this.currentTrack()?.id !== track.id) {
            this.loadingTrackId.set(null);
            return;
        }

        this.audioEngine.playSource(src)
            .then(() => {
                if (this.currentTrack()?.id === track.id) {
                    this.loadingTrackId.set(null);
                }
            })
            .catch(err => {
                this.loadingTrackId.set(null);
                if (err?.name !== 'AbortError' && this.currentTrack()?.id === track.id) {
                    this.handlePlaybackError(track, 'Playback failed');
                }
            });
    }

    public get audio(): HTMLAudioElement {
        return this.audioEngine.audio;
    }

    public setVolume(value: number): void {
        this.audioEngine.setVolume(value);
    }

    public toggleShuffle(): void {
        this.queueService.toggleShuffle(this.currentTrack());
    }

    public setQueue(tracks: PlayableTrack[]): void {
        this.queueService.setQueue(tracks, this.currentTrack());
    }

    public removeFromQueue(trackId: number): void {
        this.queueService.removeFromQueue(trackId);
        const current = this.currentTrack();
        if (current && current.id === trackId) {
            this.audioEngine.stop();
            this.currentTrack.set(null);
        }
    }

    public async play(track: PlayableTrack): Promise<void> {
        if (this.currentTrack()?.id === track.id) {
            this.togglePlay();
            return;
        }

        this.audioEngine.stop();
        this.currentTrack.set(track);
        this.loadingTrackId.set(track.id);

        this.queueService.onTrackSelected(track);

        if (track.streamUrl) {
            return this.playSource(track, track.streamUrl);
        }

        const streamUrl = track.permalinkUrl
            ? `/api/stream/${track.id}?url=${encodeURIComponent(track.permalinkUrl)}`
            : `/api/stream/${track.id}`;
        track.streamUrl = streamUrl;
        this.playSource(track, streamUrl);
    }

    public togglePlay(): void {
        if (this.isPlaying()) {
            this.audioEngine.pause();
        } else if (this.audio.src) {
            this.audioEngine.audio.play().catch(() => { });
        }
    }

    public isTrackPlaying(trackId: number): boolean;
    public isTrackPlaying(): boolean;
    public isTrackPlaying(trackId?: number): boolean {
        const current = this.currentTrack();
        if (trackId !== undefined) {
            return this.isPlaying() && current?.id === trackId;
        }
        return this.isPlaying();
    }

    public seek(time: number): void {
        this.audioEngine.seek(time);
    }

    public next(): void {
        const nextTrack = this.queueService.getNextTrack(this.currentTrack());
        if (nextTrack) {
            this.play(nextTrack);
        }
    }

    public previous(): void {
        const current = this.currentTrack();
        if (!current) return;

        if (this.audio.currentTime > 3) {
            this.seek(0);
            return;
        }

        const prevTrack = this.queueService.getPreviousTrack(current);
        if (prevTrack) {
            this.play(prevTrack);
        }
    }
}
