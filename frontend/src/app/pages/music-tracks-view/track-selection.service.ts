import { computed, Injectable, signal } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class TrackSelection {
    private readonly _ids = signal<number[]>([]);

    public readonly ids = this._ids.asReadonly();
    public readonly count = computed(() => this._ids().length);
    public readonly hasSelection = computed(() => this._ids().length > 0);

    public has(id: number): boolean {
        return this._ids().includes(id);
    }

    public select(id: number): void {
        this._ids.update((ids) => (ids.includes(id) ? ids : [...ids, id]));
    }

    public deselect(id: number): void {
        this._ids.update((ids) => ids.filter((i) => i !== id));
    }

    public toggle(id: number): void {
        this._ids.update((ids) => (ids.includes(id) ? ids.filter((i) => i !== id) : [...ids, id]));
    }

    public remove(ids: number[]): void {
        this._ids.update((current) => current.filter((id) => !ids.includes(id)));
    }

    public clear(): void {
        this._ids.set([]);
    }
}
