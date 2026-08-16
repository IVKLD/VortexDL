import { signal } from '@angular/core';
import { PlayableTrack } from '@shared/models/music-track.model';
import { shuffleArray } from '@shared/utils/array.utils';

export class PlayerQueueService {
    private readonly _queue = signal<PlayableTrack[]>([]);
    private readonly _shuffle = signal<boolean>(
        localStorage.getItem('player_shuffle') === 'true'
    );

    private shuffleQueue: PlayableTrack[] = [];
    private shuffleIndex = -1;

    public readonly queue = this._queue.asReadonly();
    public readonly shuffle = this._shuffle.asReadonly();

    public setQueue(tracks: PlayableTrack[], currentTrack: PlayableTrack | null): void {
        this._queue.set(tracks);
        if (this._shuffle()) {
            this.generateShuffleQueue(currentTrack);
        }
    }

    public removeFromQueue(trackId: number): void {
        this._queue.update(q => q.filter(t => t.id !== trackId));
        if (this._shuffle()) {
            this.shuffleQueue = this.shuffleQueue.filter(t => t.id !== trackId);
        }
    }

    public toggleShuffle(currentTrack: PlayableTrack | null): boolean {
        const newVal = !this._shuffle();
        this._shuffle.set(newVal);
        localStorage.setItem('player_shuffle', newVal.toString());
        if (newVal) {
            this.generateShuffleQueue(currentTrack);
        } else {
            this.shuffleQueue = [];
            this.shuffleIndex = -1;
        }
        return newVal;
    }

    public generateShuffleQueue(current: PlayableTrack | null): void {
        const q = shuffleArray(this._queue());
        if (q.length === 0) {
            this.shuffleQueue = [];
            this.shuffleIndex = -1;
            return;
        }

        if (current) {
            const idx = q.findIndex(t => t.id === current.id);
            if (idx !== -1) {
                q.splice(idx, 1);
                q.unshift(current);
            }
        }
        this.shuffleQueue = q;
        this.shuffleIndex = current ? 0 : -1;
    }

    public onTrackSelected(track: PlayableTrack): void {
        if (this._shuffle()) {
            const idx = this.shuffleQueue.findIndex(t => t.id === track.id);
            if (idx !== -1) {
                this.shuffleIndex = idx;
            } else {
                this.generateShuffleQueue(track);
            }
        }
    }

    public getNextTrack(currentTrack: PlayableTrack | null): PlayableTrack | null {
        const q = this._queue();
        if (q.length === 0 || !currentTrack) return null;

        if (this._shuffle()) {
            if (this.shuffleQueue.length === 0) {
                this.generateShuffleQueue(currentTrack);
            }
            if (this.shuffleQueue.length > 0) {
                this.shuffleIndex = (this.shuffleIndex + 1) % this.shuffleQueue.length;
                return this.shuffleQueue[this.shuffleIndex] ?? null;
            }
            return null;
        }

        const currentIndex = q.findIndex(t => t.id === currentTrack.id);
        const nextIndex = (currentIndex + 1) % q.length;
        return q[nextIndex] ?? null;
    }

    public getPreviousTrack(currentTrack: PlayableTrack | null): PlayableTrack | null {
        const q = this._queue();
        if (q.length === 0 || !currentTrack) return null;

        if (this._shuffle()) {
            if (this.shuffleQueue.length === 0) {
                this.generateShuffleQueue(currentTrack);
            }
            if (this.shuffleQueue.length > 0) {
                this.shuffleIndex = (this.shuffleIndex - 1 + this.shuffleQueue.length) % this.shuffleQueue.length;
                return this.shuffleQueue[this.shuffleIndex] ?? null;
            }
            return null;
        }

        const currentIndex = q.findIndex(t => t.id === currentTrack.id);
        const prevIndex = (currentIndex - 1 + q.length) % q.length;
        return q[prevIndex] ?? null;
    }
}
