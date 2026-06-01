import { effect, Injectable, inject, NgZone, signal } from '@angular/core';
import { Track } from '@shared/models/track.model';

@Injectable({
    providedIn: 'root'
})
export class PlayerService {
    private readonly _audio = new Audio();
    private readonly zone = inject(NgZone);

    public get audio(): HTMLAudioElement {
        return this._audio;
    }

    public readonly currentTrack = signal<Track | null>(null);
    private readonly _queue = signal<Track[]>([]);
    public readonly queue = this._queue.asReadonly();

    public readonly isPlaying = signal(false);
    public readonly progress = signal(0);
    public readonly duration = signal(0);

    private readonly _volume = signal<number>(
        +(localStorage.getItem('player_volume') ?? 0.2)
    );

    public readonly volume = this._volume.asReadonly()

    public setVolume(value: number): void {
        this._volume.set(value);
    }

    public setQueue(tracks: Track[]): void {
        this._queue.set(tracks);
    }

    public removeFromQueue(trackId: number): void {
        this._queue.update(q => q.filter(t => t.id !== trackId));
        const current = this.currentTrack();
        if (current && current.id === trackId) {
            this.audio.pause();
            this.audio.src = '';
            this.currentTrack.set(null);
            this.isPlaying.set(false);
        }
    }

    constructor() {
        this.listenToAudioEvents();
        this.setupMediaSession();

        effect(() => {
            const vol = this.volume();
            this.audio.volume = vol;
            localStorage.setItem('player_volume', vol.toString());
        });

        effect(() => {
            const track = this.currentTrack();
            if (track) {
                this.updateMediaSessionMetadata(track);
            }
        });
    }

    public play(track: Track): void {
        if (this.currentTrack()?.id === track.id) {
            this.togglePlay();
            return;
        }

        this.currentTrack.set(track);
        this.audio.src = `/api/downloads/${track.id}/stream`;
        this.audio.load();
        this.audio.play();
    }

    public togglePlay(): void {
        if (this.isPlaying()) {
            this.audio.pause();
        } else if (this.audio.src) {
            this.audio.play();
        }
    }

    public seek(time: number): void {
        this.audio.currentTime = time;
    }

    public next(): void {
        const queue = this.queue();
        const current = this.currentTrack();
        
        if (queue.length === 0 || !current) return;

        const currentIndex = queue.findIndex(t => t.id === current.id);
        const nextIndex = (currentIndex + 1) % queue.length;
        const nextTrack = queue[nextIndex];
        
        if (nextTrack) this.play(nextTrack);
    }

    public previous(): void {
        const current = this.currentTrack();
        
        if (!current) return;

        if (this.audio.currentTime > 3) {
            this.seek(0);
            return;
        }

        const queue = this.queue();
        if (queue.length === 0) return;

        const currentIndex = queue.findIndex(t => t.id === current.id);
        const prevIndex = (currentIndex - 1 + queue.length) % queue.length;
        const prevTrack = queue[prevIndex];
        
        if (prevTrack) this.play(prevTrack);
    }

    private listenToAudioEvents(): void {
        this.zone.runOutsideAngular(() => {
            this.audio.addEventListener('timeupdate', () => {
                this.progress.set(this.audio.currentTime);
            });
            
            this.audio.addEventListener('loadedmetadata', () => {
                this.duration.set(this.audio.duration);
            });
            
            this.audio.addEventListener('play', () => {
                this.setPlaybackState(true);
            });
            
            this.audio.addEventListener('pause', () => {
                this.setPlaybackState(false);
            });
            
            this.audio.addEventListener('ended', () => {
                this.zone.run(() => {
                    this.next();
                });
            });
        });
    }

    private setPlaybackState(playing: boolean): void {
        this.isPlaying.set(playing);
        if ('mediaSession' in navigator) {
            navigator.mediaSession.playbackState = playing ? 'playing' : 'paused';
        }
    }

    private setupMediaSession(): void {
        if (!('mediaSession' in navigator)) return;

        navigator.mediaSession.setActionHandler('play', () => this.togglePlay());
        navigator.mediaSession.setActionHandler('pause', () => this.togglePlay());
        navigator.mediaSession.setActionHandler('previoustrack', () => this.previous());
        navigator.mediaSession.setActionHandler('nexttrack', () => this.next());
    }

    private updateMediaSessionMetadata(track: Track): void {
        if (!('mediaSession' in navigator)) return;

        navigator.mediaSession.metadata = new MediaMetadata({
            title: track.title,
            artist: track.artist,
            album: 'VortexDL',
            artwork: track.artworkUrl ? [{ src: track.artworkUrl }] : []
        });
    }
}
