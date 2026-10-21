import { computed, inject, Injectable, signal } from '@angular/core';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { DownloadTrackingService } from '@app/services/download-tracking.service';
import { NotificationService } from '@app/services/notification.service';
import { AudioFormat } from '@shared/models/music-track.model';
import { FileSizePipe } from '@shared/pipes/file-size.pipe';
import {
    ActivityDay,
    DeletedTrack,
    FormatItem,
    LibraryStats,
} from './dashboard-view.model';
import { DashboardViewService } from './dashboard-view.service';

const FORMATS_CONFIG: { format: AudioFormat; color: string }[] = [
    { format: AudioFormat.MP3, color: '#818cf8' },
    { format: AudioFormat.FLAC, color: '#34d399' },
    { format: AudioFormat.WAV, color: '#f472b6' },
    { format: AudioFormat.M4A, color: '#fbbf24' },
    { format: AudioFormat.AAC, color: '#38bdf8' },
    { format: AudioFormat.OGG, color: '#a3e635' },
    { format: AudioFormat.OPUS, color: '#c084fc' },
    { format: AudioFormat.WMA, color: '#fb7185' },
    { format: AudioFormat.ALAC, color: '#2dd4bf' },
    { format: AudioFormat.AIFF, color: '#e879f9' },
    { format: AudioFormat.UNKNOWN, color: '#94a3b8' },
];

@Injectable({ providedIn: 'root' })
export class DashboardViewState {
    private readonly _api = inject(DashboardViewService);
    private readonly _musicState = inject(MusicTracksViewState);
    private readonly _notification = inject(NotificationService);
    private readonly _tracking = inject(DownloadTrackingService);
    private readonly _fileSize = new FileSizePipe();

    public readonly serverStats = signal<LibraryStats | null>(null);
    public readonly loading = signal<boolean>(false);

    public readonly recentDeleted = computed<DeletedTrack[]>(() => this.serverStats()?.recentDeleted ?? []);
    public readonly deletedCount = computed<number>(() => this.serverStats()?.totalDeleted ?? 0);
    public readonly deletedBytes = computed<number>(() => this.serverStats()?.totalDeletedBytes ?? 0);

    public readonly totalTracks = computed<string>(() => this._musicState.sortedTracks().length.toString());
    public readonly totalSize = computed<string>(() => {
        const bytes = this._musicState.sortedTracks().reduce((acc, t) => acc + (t.size || 0), 0);
        return this._fileSize.transform(bytes);
    });
    public readonly totalDownloaded = computed<string>(() => {
        const count = this.serverStats()?.totalDownloaded ?? this._musicState.sortedTracks().length;
        return count.toString();
    });
    public readonly totalDownloadedBytes = computed<string>(() => {
        const bytes = this.serverStats()?.totalDownloadedBytes ??
            this._musicState.sortedTracks().reduce((acc, t) => acc + (t.size || 0), 0);
        return this._fileSize.transform(bytes);
    });
    public readonly deletedTracks = computed<string>(() => this.deletedCount().toString());
    public readonly spaceRecovered = computed<string>(() => this._fileSize.transform(this.deletedBytes()));
    public readonly activeQueue = computed<string>(() => this._tracking.activeDownloads().length.toString());

    public readonly formatBreakdown = computed<FormatItem[]>(() => {
        const tracks = this._musicState.sortedTracks();
        if (tracks.length === 0) return [];

        return FORMATS_CONFIG.map((config) => {
            const matching = tracks.filter((t) => (t.format || AudioFormat.UNKNOWN) === config.format);
            const count = matching.length;
            const sizeBytes = matching.reduce((acc, t) => acc + (t.size || 0), 0);
            return {
                format: config.format,
                color: config.color,
                count,
                percentage: (count / tracks.length) * 100,
                sizeBytes,
            };
        }).filter((item) => item.count > 0).sort((a, b) => b.count - a.count);
    });

    public readonly activityData = computed<ActivityDay[]>(() => {
        const tracks = this._musicState.sortedTracks();
        const deleted = this.recentDeleted();

        const now = new Date();
        now.setHours(0, 0, 0, 0);

        const downloadCounts = new Array(7).fill(0);
        const deleteCounts = new Array(7).fill(0);

        for (const t of tracks) {
            if (!t.createdAt) continue;
            const trackDate = new Date(t.createdAt * 1000);
            trackDate.setHours(0, 0, 0, 0);
            const diffDays = Math.floor((now.getTime() - trackDate.getTime()) / 86400000);
            if (diffDays >= 0 && diffDays < 7) {
                downloadCounts[6 - diffDays]++;
            }
        }

        for (const d of deleted) {
            if (!d.deletedAt) continue;
            const delDate = new Date(d.deletedAt * 1000);
            delDate.setHours(0, 0, 0, 0);
            const diffDays = Math.floor((now.getTime() - delDate.getTime()) / 86400000);
            if (diffDays >= 0 && diffDays < 7) {
                deleteCounts[6 - diffDays]++;
            }
        }

        const max = Math.max(...downloadCounts, ...deleteCounts, 1);

        return downloadCounts.map((dlCount, i) => {
            const delCount = deleteCounts[i];
            const date = new Date(now.getTime() - (6 - i) * 86400000);
            return {
                label: date.toLocaleDateString('en-US', { weekday: 'short' }),
                downloadCount: dlCount,
                deleteCount: delCount,
                downloadHeightPercent: dlCount > 0 ? (dlCount / max) * 100 : 0,
                deleteHeightPercent: delCount > 0 ? (delCount / max) * 100 : 0,
            };
        });
    });

    public loadStats(): void {
        this.loading.set(true);
        this._api.getStats().subscribe({
            next: (data) => {
                this.serverStats.set(data);
                this.loading.set(false);
            },
            error: () => {
                this.loading.set(false);
                this._notification.error('Failed to load dashboard statistics');
            },
        });
    }

    public clearDeletedHistory(): void {
        this._api.clearDeletedHistory().subscribe({
            next: () => {
                this.loadStats();
                this._notification.success('Deleted tracks history cleared');
            },
            error: () => {
                this._notification.error('Failed to clear deleted tracks history');
            },
        });
    }
}
