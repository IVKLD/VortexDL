import { inject, Injectable, NgZone, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { MusicTracksViewState } from '../pages/music-tracks-view/music-tracks-view.state';
import { PlayerService } from '@app/services/player.service';
import { AudioFormat, MusicTrack } from '@shared/models/music-track.model';
import { Subscription } from 'rxjs';
import { WebSocketService } from './websocket.service';

export enum DownloadStatus {
    Queued = 'queued',
    Downloading = 'downloading',
    Finished = 'finished',
    Failed = 'failed',
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
export class DownloadTrackingService {
    private readonly _http = inject(HttpClient);
    private readonly _musicState = inject(MusicTracksViewState);
    private readonly _player = inject(PlayerService);
    private readonly _zone = inject(NgZone);
    private readonly _wsService = inject(WebSocketService);

    private _wsSubscription?: Subscription;
    public readonly activeDownloads = signal<DownloadItem[]>([]);
    public readonly errors = signal<string[]>([]);
    public readonly syncingUrls = signal<string[]>([]);

    constructor() {
        this.syncActiveDownloads();
        this.fetchSyncingUrls();
        this.initializeWebSocket();
    }

    private syncActiveDownloads(): void {
        this._http.get<DownloadItem[]>('/download/queue').subscribe({
            next: items => this.activeDownloads.set(items),
            error: err => console.error('Failed to sync active downloads:', err)
        });
    }

    private fetchSyncingUrls(): void {
        this._http.get<string[]>('/download/syncing').subscribe({
            next: urls => this.updateSyncingUrls(urls),
            error: err => console.error('Failed to sync active sync processes:', err)
        });
    }

    private updateSyncingUrls(urls: string[]): void {
        this.syncingUrls.set(urls);
    }

    private initializeWebSocket(): void {
        this.cleanupWebSocket();

        this._wsSubscription = this._wsService.connect<ServerEvent>('/api/download/events').subscribe({
            next: (serverEvent) => this.handleServerEvent(serverEvent),
            error: (err) => console.error('Download tracking WebSocket error:', err),
        });
    }

    private cleanupWebSocket(): void {
        if (this._wsSubscription) {
            this._wsSubscription.unsubscribe();
            this._wsSubscription = undefined;
        }
    }

    private handleServerEvent(event: ServerEvent): void {
        switch (event.type) {
            case ServerEventType.TrackUpdate:
                this.handleTrackUpdate(event.item);
                break;
            case ServerEventType.SyncStarted:
                this.updateSyncingUrls([...this.syncingUrls().filter(u => u !== event.url), event.url]);
                break;
            case ServerEventType.SyncFinished:
                if (event.url) {
                    this.updateSyncingUrls(this.syncingUrls().filter(u => u !== event.url));
                } else {
                    this.fetchSyncingUrls();
                }
                this._musicState.refresh().subscribe({
                    next: () => this.syncPlayerQueue()
                });
                break;
            case ServerEventType.Message:
                if (event.level === 'error') this.addError(event.message);
                break;
            case ServerEventType.Error:
                this.addError(event.message);
                break;
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
        } else if (item.status === DownloadStatus.Failed) {
            this.addError(`Failed to download "${item.artist} - ${item.title}": ${item.error || 'Unknown error'}`);
        }
    }

    private addError(message: string): void {
        this.errors.update(prev => prev.includes(message) ? prev : [message, ...prev].slice(0, 5));
    }

    private syncPlayerQueue(): void {
        const validIds = new Set(this._musicState.sortedTracks().map(t => t.id));
        for (const track of this._player.queue()) {
            if (!validIds.has(track.id)) {
                this._player.removeFromQueue(track.id);
            }
        }
    }

    private updateActiveDownloads(item: DownloadItem): void {
        this.activeDownloads.update(downloads => {
            const filtered = downloads.filter(d => d.id !== item.id);
            if (item.status === DownloadStatus.Finished || item.status === DownloadStatus.Failed) {
                return filtered;
            }
            return [...filtered, item];
        });
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
}

