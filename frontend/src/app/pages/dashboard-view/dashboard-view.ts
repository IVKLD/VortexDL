import { ChangeDetectionStrategy, Component, computed, inject, OnInit } from '@angular/core';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { DownloadTrackingService } from '@app/services/download-tracking.service';
import { ActiveDownloadsComponent } from '@shared/components/active-downloads/active-downloads';
import { MatIcon } from '@angular/material/icon';
import { RouterLink } from '@angular/router';
import { FileSizePipe } from '@shared/pipes/file-size.pipe';
import { StatCardComponent } from './components/stat-card/stat-card.component';
import { ActivityChartComponent } from './components/activity-chart/activity-chart.component';
import { FormatBreakdownComponent } from './components/format-breakdown/format-breakdown.component';
import { RecentTracksComponent } from './components/recent-tracks/recent-tracks.component';

const FORMAT_COLORS: Record<string, string> = { MP3: '#818cf8', FLAC: '#34d399', WAV: '#f472b6' };

@Component({
    selector: 'app-dashboard-view',
    imports: [
        ActiveDownloadsComponent,
        MatIcon,
        RouterLink,
        FileSizePipe,
        StatCardComponent,
        ActivityChartComponent,
        FormatBreakdownComponent,
        RecentTracksComponent
    ],
    templateUrl: './dashboard-view.html',
    styleUrl: './dashboard-view.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardView implements OnInit {
    private readonly _state = inject(MusicTracksViewState);
    private readonly _api = inject(MusicTracksViewService);
    public readonly tracking = inject(DownloadTrackingService);

    public readonly stats = computed(() => [
        { icon: 'library_music', label: 'Total Tracks', value: this._state.sortedTracks().length, iconClass: 'track-icon' },
        { icon: 'storage', label: 'Total Library Size', value: this._state.sortedTracks().reduce((acc, t) => acc + (t.size || 0), 0), iconClass: 'size-icon', isSize: true },
        { icon: 'downloading', label: 'Active Downloads', value: this.tracking.activeDownloads().length, iconClass: 'active-icon' }
    ]);

    public readonly recentTracks = computed(() => [...this._state.sortedTracks()].sort((a, b) => b.createdAt - a.createdAt).slice(0, 5));

    public readonly formatBreakdown = computed(() => {
        const tracks = this._state.sortedTracks();
        const counts = tracks.reduce((acc, t) => {
            const fmt = (t.format || 'unknown').toUpperCase();
            acc[fmt] = (acc[fmt] || 0) + 1;
            return acc;
        }, {} as Record<string, number>);

        return Object.entries(counts).map(([format, count]) => ({
            format, count, percentage: (count / tracks.length) * 100, color: FORMAT_COLORS[format] || '#94a3b8'
        })).sort((a, b) => b.count - a.count);
    });

    public readonly activityData = computed(() => {
        const tracks = this._state.sortedTracks();
        const now = new Date(); now.setHours(0, 0, 0, 0);
        const dayCounts = new Array(7).fill(0);

        tracks.forEach(t => {
            const diff = Math.floor((now.getTime() - new Date(t.createdAt * 1000).setHours(0, 0, 0, 0)) / 86400000);
            if (diff >= 0 && diff < 7) dayCounts[6 - diff]++;
        });

        const max = Math.max(...dayCounts, 1);
        return dayCounts.map((count, i) => ({
            label: new Date(now.getTime() - (6 - i) * 86400000).toLocaleDateString('en-US', { weekday: 'short' }),
            count, heightPercent: (count / max) * 100 || 5
        }));
    });

    ngOnInit() {
        if (this._state.isLoading()) this._api.getAll().subscribe(t => this._state.setTracks = t);
    }
}
