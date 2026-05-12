import { computed, inject, Injectable, NgZone, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { MusicTracksViewService } from '../pages/music-tracks-view/music-tracks-view.service';
import { MusicTracksViewState } from '../pages/music-tracks-view/music-tracks-view.state';
import { AudioFormat } from '@shared/models/track.model';

export enum DownloadStatus {
    Queued = 'queued',
    Downloading = 'downloading',
    Finished = 'finished',
    Failed = 'failed',
}

export interface DownloadItem {
    id: number;
    title: string;
    artist: string;
    status: DownloadStatus;
    artworkUrl?: string | null;
    format?: AudioFormat;
    createdAt?: number;
    sourceUrl?: string | null;
    error?: string;
    size?: number;
}

export enum ServerEventType {
    TrackUpdate = 'trackupdate',
    SyncFinished = 'syncfinished',
    Error = 'error',
    Message = 'message',
}

export type ServerEvent =
    | { type: ServerEventType.TrackUpdate; item: DownloadItem }
    | { type: ServerEventType.SyncFinished }
    | { type: ServerEventType.Error; message: string }
    | { type: ServerEventType.Message; message: string; level: string };

@Injectable({
    providedIn: 'root',
})
export class DownloadTrackingService {
    public readonly activeDownloads = signal<DownloadItem[]>([]);
    public readonly sortedActiveDownloads = computed(() => {
        return [...this.activeDownloads()].sort((a, b) => {
            if (a.status === DownloadStatus.Downloading && b.status !== DownloadStatus.Downloading) return -1;
            if (a.status !== DownloadStatus.Downloading && b.status === DownloadStatus.Downloading) return 1;
            return 0;
        });
    });
    public readonly errors = signal<string[]>([]);
    private readonly _http = inject(HttpClient);
    private readonly _musicApi = inject(MusicTracksViewService);
    private readonly _musicState = inject(MusicTracksViewState);
    private readonly _zone = inject(NgZone);

    constructor() {
        this.syncActiveDownloads();
        this.initializeEventSource();
    }

    public removeFromQueue(id: number): void {
        this._http.delete(`/download/queue/${id}`).subscribe({
            next: () => this.activeDownloads.update(items => items.filter(i => i.id !== id)),
            error: err => {
                console.error('Failed to remove from queue:', err);
                this.addError('Failed to remove track from queue.');
            },
        });
    }

    public clearError(): void {
        this.errors.set([]);
    }

    private syncActiveDownloads(): void {
        this._http.get<DownloadItem[]>('/download/queue').subscribe({
            next: items => this.activeDownloads.set(items),
            error: err => console.error('Failed to sync active downloads:', err)
        });
    }

    private initializeEventSource(): void {
        const eventSource = new EventSource('/api/download/events');

        eventSource.onopen = () => {
            console.info('SSE connection established');
        };

        eventSource.onmessage = event => {
            this._zone.run(() => {
                try {
                    const serverEvent: ServerEvent = JSON.parse(event.data);
                    this.handleServerEvent(serverEvent);
                } catch (e) {
                    console.error('Failed to parse SSE event:', e, event.data);
                }
            });
        };

        eventSource.onerror = error => {
            this._zone.run(() => {
                console.error('SSE Error, attempting to reconnect:', error);
                eventSource?.close();

                setTimeout(() => this.initializeEventSource(), 3000);
            });
        };
    }

    private handleServerEvent(event: ServerEvent): void {
        switch (event.type) {
            case ServerEventType.TrackUpdate:
                this.handleTrackUpdate(event.item);
                break;
            case ServerEventType.SyncFinished:
                this.refreshMusicList();
                break;
            case ServerEventType.Message:
                this.handleMessage(event.message, event.level);
                break;
            case ServerEventType.Error:
                this.addError(event.message);
                break;
        }
    }

    private handleMessage(message: string, level: string): void {
        if (level === 'error') {
            this.addError(message);
        }
    }

    private handleTrackUpdate(item: DownloadItem): void {
        this.updateActiveDownloads(item);

        if (item.status === DownloadStatus.Finished) {
            this._musicState.addTrack({
                id: item.id,
                artist: item.artist,
                title: item.title,
                format: item.format || AudioFormat.MP3,
                artworkUrl: item.artworkUrl || null,
                sourceUrl: item.sourceUrl || null,
                createdAt: item.createdAt || 0,
                size: item.size || 0,
            });
        }

        if (item.status === DownloadStatus.Failed) {
            this.addError(`Failed to download "${item.artist} - ${item.title}": ${item.error || 'Unknown error'}`);
        }
    }

    private addError(message: string): void {
        this.errors.update(prev => {
            if (prev.includes(message)) return prev;
            return [message, ...prev].slice(0, 5);
        });
    }

    private updateActiveDownloads(item: DownloadItem): void {
        this.activeDownloads.update(downloads => {
            if (item.status === DownloadStatus.Finished || item.status === DownloadStatus.Failed) {
                return downloads.filter(d => d.id !== item.id);
            }

            const index = downloads.findIndex(d => d.id === item.id);
            if (index !== -1) {
                const newDownloads = [...downloads];
                newDownloads[index] = item;
                return newDownloads;
            } else {
                return [...downloads, item];
            }
        });
    }

    private refreshMusicList(): void {
        this._musicApi.getAll().subscribe({
            next: tracks => (this._musicState.setTracks = tracks),
        });
    }
}

