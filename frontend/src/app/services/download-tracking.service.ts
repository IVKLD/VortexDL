import {computed, inject, Injectable, NgZone, signal} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {MusicTracksViewService} from '../pages/music-tracks-view/music-tracks-view.service';
import {MusicTracksViewState} from '../pages/music-tracks-view/music-tracks-view.state';

export enum DownloadStatus {
    Queued = 'queued',
    Downloading = 'downloading',
    Finished = 'finished',
    Failed = 'failed',
}

export interface DownloadItem {
    id: number;
    title: string;
    status: DownloadStatus;
    artwork_url?: string;
    format?: 'mp3' | 'wav' | 'flac';
    created_at?: number;
    source_url?: string;
    error?: string;
}

export enum ServerEventType {
    TrackUpdate = 'trackupdate',
    SyncFinished = 'syncfinished',
    Error = 'error',
    Message = 'message',
}

export type ServerEvent =
    | { type: ServerEventType.TrackUpdate, item: DownloadItem }
    | { type: ServerEventType.SyncFinished }
    | { type: ServerEventType.Error, message: string }
    | { type: ServerEventType.Message, message: string, level: string };

@Injectable({
    providedIn: 'root'
})
export class DownloadTrackingService {
    private readonly _http = inject(HttpClient);
    private readonly _musicApi = inject(MusicTracksViewService);
    private readonly _musicState = inject(MusicTracksViewState);
    private readonly _zone = inject(NgZone);

    public readonly activeDownloads = signal<DownloadItem[]>([]);
    public readonly sortedActiveDownloads = computed(() => {
        return [...this.activeDownloads()].sort((a, b) => {
            if (a.status === DownloadStatus.Downloading && b.status !== DownloadStatus.Downloading) return -1;
            if (a.status !== DownloadStatus.Downloading && b.status === DownloadStatus.Downloading) return 1;
            return 0;
        });
    });
    public readonly errors = signal<string[]>([]);

    constructor() {
        this.initializeEventSource();
    }

    private initializeEventSource(): void {
        const eventSource = new EventSource(`${process.env.NG_APP_API_URL}/download/events`);

        eventSource.onmessage = (event) => {
            this._zone.run(() => {
                try {
                    const serverEvent: ServerEvent = JSON.parse(event.data);
                    this.handleServerEvent(serverEvent);
                } catch (e) {
                    console.error('Failed to parse SSE event:', e, event.data);
                }
            });
        };

        eventSource.onerror = (error) => {
            console.error('SSE Error:', error);
            this.addError('Connection lost. Real-time updates may be unavailable.');
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

    public removeFromQueue(id: number): void {
        this._http.delete(`/download/queue/${id}`).subscribe({
            next: () => this.activeDownloads.update(items => items.filter(i => i.id !== id)),
            error: (err) => {
                console.error('Failed to remove from queue:', err);
                this.addError('Failed to remove track from queue.');
            }
        });
    }

    private handleTrackUpdate(item: DownloadItem): void {
        this.updateActiveDownloads(item);

        if (item.status === DownloadStatus.Finished) {
            this._musicState.addTrack({
                id: item.id,
                filename: item.title,
                album: '',
                format: item.format || 'mp3',
                artwork_url: item.artwork_url,
                source_url: item.source_url,
                created_at: item.created_at || Math.floor(Date.now() / 1000)
            });
        }

        if (item.status === DownloadStatus.Failed) {
            this.addError(`Failed to download "${item.title}": ${item.error || 'Unknown error'}`);
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
            if (item.status === DownloadStatus.Finished) {
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

    public clearError(): void {
        this.errors.set([]);
    }

    private refreshMusicList(): void {
        this._musicApi.getAll().subscribe({
            next: (tracks) => this._musicState.setTracks = tracks
        });
    }
}
