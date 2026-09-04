import { effect, inject, NgZone, signal } from '@angular/core';

export class AudioEngineService {
    private readonly zone = inject(NgZone);
    private readonly _audio = new Audio();

    private readonly _volume = signal<number>(
        +(localStorage.getItem('player_volume') ?? 0.2)
    );

    public readonly isPlaying = signal(false);
    public readonly progress = signal(0);
    public readonly duration = signal(0);
    public readonly volume = this._volume.asReadonly();

    public onEnded?: () => void;
    public onError?: (err: unknown) => void;

    constructor() {
        this.listenToAudioEvents();

        effect(() => {
            const vol = this.volume();
            this._audio.volume = Math.pow(vol, 2);
            localStorage.setItem('player_volume', vol.toString());
        });
    }

    private listenToAudioEvents(): void {
        this.zone.runOutsideAngular(() => {
            this._audio.addEventListener('timeupdate', () => {
                this.progress.set(this._audio.currentTime);
            });

            this._audio.addEventListener('loadedmetadata', () => {
                this.duration.set(this._audio.duration);
            });

            this._audio.addEventListener('play', () => {
                this.isPlaying.set(true);
            });

            this._audio.addEventListener('pause', () => {
                this.isPlaying.set(false);
            });

            this._audio.addEventListener('ended', () => this.zone.run(() => this.onEnded?.()));
            this._audio.addEventListener('error', err => this.zone.run(() => this.onError?.(err)));
        });
    }

    public get audio(): HTMLAudioElement {
        return this._audio;
    }

    public setVolume(value: number): void {
        this._volume.set(value);
    }

    public toggleMute(): void {
        if (this._volume() > 0) {
            this._volume.set(0);
        } else {
            this._volume.set(0.2);
        }
    }

    public seek(time: number): void {
        this._audio.currentTime = time;
    }

    public pause(): void {
        this._audio.pause();
    }

    public stop(): void {
        this._audio.pause();
        this._audio.removeAttribute('src');
        this._audio.load();
        this.isPlaying.set(false);
    }

    public async playSource(src: string): Promise<void> {
        this._audio.src = src;
        this._audio.load();
        await this._audio.play();
    }
}
