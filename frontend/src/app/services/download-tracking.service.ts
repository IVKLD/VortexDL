import { inject, Injectable, NgZone, signal } from '@angular/core';
import { NotificationService } from './notification.service';
import { MusicTracksViewService } from '../pages/music-tracks-view/music-tracks-view.service';
import { MusicTracksViewState } from '../pages/music-tracks-view/music-tracks-view.state';

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
}

export enum ServerEventType {
  TrackUpdate = 'trackupdate',
  OperationStarted = 'operationstarted',
  OperationFinished = 'operationfinished',
  Error = 'error',
  Message = 'message',
}

export type ServerEvent = 
  | { type: ServerEventType.TrackUpdate, item: DownloadItem }
  | { type: ServerEventType.OperationStarted, url: string, kind: string }
  | { type: ServerEventType.OperationFinished, url: string, kind: string, status: string }
  | { type: ServerEventType.Error, message: string }
  | { type: ServerEventType.Message, message: string, level: string };

@Injectable({
  providedIn: 'root'
})
export class DownloadTrackingService {
  private readonly _notification = inject(NotificationService);
  private readonly _musicApi = inject(MusicTracksViewService);
  private readonly _musicState = inject(MusicTracksViewState);
  private readonly _zone = inject(NgZone);

  public readonly activeDownloads = signal<DownloadItem[]>([]);
  private _dismissTimeout?: any;

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
    };
  }

  private handleServerEvent(event: ServerEvent): void {
    switch (event.type) {
      case ServerEventType.TrackUpdate:
        this.handleTrackUpdate(event.item);
        break;
      case ServerEventType.OperationStarted:
        this._notification.showInfo(`Operation started: ${event.kind}`);
        break;
      case ServerEventType.OperationFinished:
        if (event.status === DownloadStatus.Finished) {
          this._notification.showSuccess(`Operation finished: ${event.kind}`);
        } else {
          this._notification.showError(`Operation failed: ${event.kind}`);
        }
        break;
      case ServerEventType.Message:
        this.handleMessage(event.message, event.level);
        break;
      case ServerEventType.Error:
        this._notification.showError(event.message);
        break;
    }
  }

  private handleTrackUpdate(item: DownloadItem): void {
    this.updateActiveDownloads(item);

    if (item.status === DownloadStatus.Finished) {
      this.refreshMusicList();
    }

    this.updateNotification();
  }

  private handleMessage(message: string, level: string): void {
    switch (level) {
      case 'success':
        this._notification.showSuccess(message);
        break;
      case 'error':
        this._notification.showError(message);
        break;
      case 'info':
      default:
        this._notification.showInfo(message);
        break;
    }
  }

  private updateActiveDownloads(item: DownloadItem): void {
    this.activeDownloads.update(downloads => {
      const index = downloads.findIndex(d => d.id === item.id);
      if (item.status === DownloadStatus.Finished || item.status === DownloadStatus.Failed) {
        return downloads.filter(d => d.id !== item.id);
      }
      if (index !== -1) {
        const newDownloads = [...downloads];
        newDownloads[index] = item;
        return newDownloads;
      } else {
        return [...downloads, item];
      }
    });
  }

  private updateNotification(): void {
    const active = this.activeDownloads();
    
    if (this._dismissTimeout) {
      clearTimeout(this._dismissTimeout);
      this._dismissTimeout = undefined;
    }

    if (active.length === 0) {
      this._notification.showStatus('All downloads finished');
      this._dismissTimeout = setTimeout(() => {
        this._notification.dismissStatus();
      }, 5000);
      return;
    }

    const current = active[0];
    let message = '';
    
    if (active.length > 1) {
      message = `Downloading ${active.length} songs... Current: ${current.title}`;
    } else {
      message = `Downloading: ${current.title}`;
    }

    this._notification.showStatus(message);
  }

  private refreshMusicList(): void {
    this._musicApi.getAll().subscribe({
      next: (tracks) => {
        this._musicState.setTracks = tracks;
      }
    });
  }
}
