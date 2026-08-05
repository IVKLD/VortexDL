import { inject, Injectable, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { MusicTracksViewState } from '../pages/music-tracks-view/music-tracks-view.state';
import { PlayerService } from '@app/services/player.service';
import { AudioFormat, MusicTrack } from '@shared/models/music-track.model';
import { Observable } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { WebSocketService } from './websocket.service';

export enum DownloadStatus {
    Queued = 'queued',
    Downloading = 'downloading',
    Finished = 'finished',
    Failed = 'failed',
    Canceled = 'canceled',
}

export interface DownloadItem extends MusicTrack {
    error?: string;
    status: DownloadStatus;
    progress?: number;
}

export enum ServerEventType {
    TrackUpdate = 'trackupdate',
    SyncStarted = 'syncstarted',
    SyncFinished = 'syncfinished',
    Error = 'error',
    Message = 'message',
}

export type ServerEvent =
    | { type: ServerEventType.TrackUpdate; item: DownloadItem }
    | { type: ServerEventType.SyncStarted; url: string }
    | { type: ServerEventType.SyncFinished; url: string | null }
    | { type: ServerEventType.Error; message: string }
    | { type: ServerEventType.Message; message: string; level: string };

@Injectable({
    providedIn: 'root',
})
export class DownloadTrackingApiService {
    private readonly _http = inject(HttpClient);

    public getQueue(): Observable<DownloadItem[]> {
        return this._http.get<DownloadItem[]>('/download/queue');
    }

    public getSyncingUrls(): Observable<string[]> {
        return this._http.get<string[]>('/download/syncing');
    }

    public removeFromQueue(id: number): Observable<void> {
        return this._http.delete<void>(`/download/queue/${id}`);
    }
}

@Injectable({
    providedIn: 'root',
})
export class DownloadTrackingService {
    private readonly _api = inject(DownloadTrackingApiService);
    private readonly _musicState = inject(MusicTracksViewState);
    private readonly _player = inject(PlayerService);
    private readonly _wsService = inject(WebSocketService);

    public readonly activeDownloads = signal<DownloadItem[]>([]);
    public readonly errors = signal<string[]>([]);
    public readonly syncingUrls = signal<string[]>([]);

    constructor() {
        this.syncActiveDownloads();
        this.fetchSyncingUrls();
        this.initializeWebSocket();
    }

    private syncActiveDownloads(): void {
        this._api.getQueue().subscribe({
            next: items => this.activeDownloads.set(items),
            error: err => console.error('Failed to sync active downloads:', err),
        });
    }

    private fetchSyncingUrls(): void {
        this._api.getSyncingUrls().subscribe({
            next: urls => this.syncingUrls.set(urls),
            error: err => console.error('Failed to sync active sync processes:', err),
        });
    }

    private initializeWebSocket(): void {
        this._wsService
            .connect<ServerEvent>('/api/download/events')
            .pipe(takeUntilDestroyed())
            .subscribe({
                next: (serverEvent) => this.handleServerEvent(serverEvent),
                error: (err) => console.error('Download tracking WebSocket error:', err),
            });
    }

    private handleServerEvent(event: ServerEvent): void {
        if (event.type === ServerEventType.TrackUpdate) {
            this.handleTrackUpdate(event.item);
            return;
        }

        if (event.type === ServerEventType.SyncStarted) {
            this.syncingUrls.set([...this.syncingUrls().filter(u => u !== event.url), event.url]);
            return;
        }

        if (event.type === ServerEventType.SyncFinished) {
            if (event.url) {
                this.syncingUrls.set(this.syncingUrls().filter(u => u !== event.url));
            } else {
                this.fetchSyncingUrls();
            }
            return;
        }

        if (event.type === ServerEventType.Message) {
            if (event.level === 'error') this.addError(event.message);
            return;
        }

        if (event.type === ServerEventType.Error) {
            this.addError(event.message);
            return;
        }
    }

    private handleTrackUpdate(item: DownloadItem): void {
        if (item.status === DownloadStatus.Finished) {
            this.activeDownloads.update(downloads => downloads.filter(d => d.id !== item.id));

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
            this.syncPlayerQueue();
        } else if (item.status === DownloadStatus.Failed) {
            this.activeDownloads.update(downloads => downloads.filter(d => d.id !== item.id));
            this.addError(`Failed to download "${item.artist} - ${item.title}": ${item.error || 'Unknown error'}`);
        } else {
            this.updateActiveDownloads(item);
        }
    }

    private syncPlayerQueue(): void {
        const localTrackIds = new Set(this._musicState.sortedTracks().map(t => t.id));
        for (const track of this._player.queue()) {
            if (!track.streamUrl && !localTrackIds.has(track.id)) {
                this._player.removeFromQueue(track.id);
            }
        }
    }

    private updateActiveDownloads(item: DownloadItem): void {
        this.activeDownloads.update(downloads => {
            const index = downloads.findIndex(d => d.id === item.id);

            if (
                item.status === DownloadStatus.Canceled ||
                item.status === DownloadStatus.Failed ||
                item.status === DownloadStatus.Finished
            ) {
                return downloads.filter(d => d.id !== item.id);
            }

            if (index !== -1) {
                const updated = [...downloads];
                updated[index] = { ...updated[index], ...item };
                return updated;
            }

            return [...downloads, item];
        });
    }

    private addError(message: string): void {
        this.errors.update(prev => prev.includes(message) ? prev : [message, ...prev].slice(0, 5));
    }

    public removeFromQueue(id: number): void {
        this._api.removeFromQueue(id).subscribe({
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
}
