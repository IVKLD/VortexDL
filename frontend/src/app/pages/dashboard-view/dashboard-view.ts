import {Component, computed, inject} from '@angular/core';
import {DashboardStat, FormatItem} from './dashboard-view.model';
import {MusicTracksViewState} from '@app/pages/music-tracks-view/music-tracks-view.state';
import {DownloadTrackingService} from '@app/services/download-tracking.service';
import {ActivityChartComponent} from './components/activity-chart/activity-chart.component';
import {FormatBreakdownComponent} from './components/format-breakdown/format-breakdown.component';
import {StatCardComponent} from './components/stat-card/stat-card.component';
import {FileSizePipe} from '@shared/pipes/file-size.pipe';
import {AudioFormat} from '@shared/models/music-track.model';

const FORMATS_CONFIG: { format: AudioFormat, color: string }[] = [
    {format: AudioFormat.MP3, color: '#818cf8'},
    {format: AudioFormat.FLAC, color: '#34d399'},
    {format: AudioFormat.WAV, color: '#f472b6'},
    {format: AudioFormat.UNKNOWN, color: '#94a3b8'}
];

@Component({
    selector: 'app-dashboard-view',
    imports: [
        ActivityChartComponent,
        FormatBreakdownComponent,
        StatCardComponent,
        FileSizePipe
    ],
    templateUrl: './dashboard-view.html',
    styleUrl: './dashboard-view.scss',
    })
export class DashboardView {
    private readonly _state = inject(MusicTracksViewState);
    public readonly tracking = inject(DownloadTrackingService);

    public readonly stats = computed<DashboardStat[]>(() => [
        {
            icon: 'library_music',
            label: 'Total Tracks',
            value: this._state.sortedTracks().length,
            iconClass: 'track-icon'
        },
        {
            icon: 'storage',
            label: 'Total Library Size',
            value: this._state.sortedTracks().reduce((acc, t) => acc + (t.size || 0), 0),
            iconClass: 'size-icon',
            isSize: true
        },
        {
            icon: 'downloading',
            label: 'Active Downloads',
            value: this.tracking.activeDownloads().length,
            iconClass: 'active-icon'
        }
    ]);
    public readonly formatBreakdown = computed<FormatItem[]>(() => {
        const tracks = this._state.sortedTracks();
        if (tracks.length === 0) return [];

        return FORMATS_CONFIG.map(config => {
            const count = tracks.filter(t => (t.format || AudioFormat.UNKNOWN) === config.format).length;
            return {
                format: config.format,
                color: config.color,
                count,
                percentage: (count / tracks.length) * 100
            };
        }).filter(item => item.count > 0).sort((a, b) => b.count - a.count);
    });
    public readonly activityData = computed(() => {
        const tracks = this._state.sortedTracks();
        const now = new Date();
        now.setHours(0, 0, 0, 0);
        const dayCounts = new Array(7).fill(0);

        tracks.forEach(t => {
            const diff = Math.floor((now.getTime() - new Date(t.createdAt * 1000).setHours(0, 0, 0, 0)) / 86400000);
            if (diff >= 0 && diff < 7) dayCounts[6 - diff]++;
        });

        const max = Math.max(...dayCounts, 1);
        return dayCounts.map((count, i) => ({
            label: new Date(now.getTime() - (6 - i) * 86400000).toLocaleDateString('en-US', {weekday: 'short'}),
            count,
            heightPercent: (count / max) * 100 || 5
        }));
    });
}
