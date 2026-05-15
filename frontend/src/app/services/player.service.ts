import {effect, Injectable, signal} from '@angular/core';
import {Track} from '@shared/models/track.model';

@Injectable({
    providedIn: 'root'
})
export class PlayerService {
    private audio = new Audio();

    public readonly currentTrack = signal<Track | null>(null);
    public readonly queue = signal<Track[]>([]);
    public readonly isPlaying = signal(false);
    public readonly progress = signal(0);
    public readonly duration = signal(0);

    private readonly _volume = signal(.2);

    public readonly volume = this._volume.asReadonly()

    public setVolume(value: number): void {
        this._volume.set(value);
    }


    constructor() {
        this.listenToAudioEvents();

        effect(() => {
            this.audio.volume = this.volume();
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
        } else {
            if (this.audio.src) {
                this.audio.play();
            }
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

        const queue = this.queue();
        if (queue.length === 0) return;

        const currentIndex = queue.findIndex(t => t.id === current.id);
        const prevIndex = (currentIndex - 1 + queue.length) % queue.length;
        const prevTrack = queue[prevIndex];

        if (prevTrack) {
            this.play(prevTrack);
        }
    }

    private listenToAudioEvents(): void {
        this.audio.addEventListener('timeupdate', () => {
            this.progress.set(this.audio.currentTime);
        });

        this.audio.addEventListener('loadedmetadata', () => {
            this.duration.set(this.audio.duration);
        });

        this.audio.addEventListener('play', () => {
            this.isPlaying.set(true);
        });

        this.audio.addEventListener('pause', () => {
            this.isPlaying.set(false);
        });

        this.audio.addEventListener('ended', () => {
            this.next();
        });
    }
}
