import { ChangeDetectionStrategy, Component, computed, inject, input, signal } from '@angular/core';
import { NgTemplateOutlet } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { MatMenu, MatMenuItem, MatMenuTrigger } from '@angular/material/menu';
import { MatTooltip } from '@angular/material/tooltip';
import { MatDivider } from '@angular/material/list';
import { MatDialog } from '@angular/material/dialog';
import { MatSnackBar } from '@angular/material/snack-bar';
import { MusicSortOption, MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { SettingsService } from '@app/pages/settings-view/settings.service';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { DownloadTrackingService } from '@app/services/download-tracking.service';
import { DownloadProgressSnackbar } from '@shared/components/download-progress-snackbar/download-progress-snackbar';
import { DownloadDialogComponent } from '../../download-dialog/download-dialog.component';
import { HeaderConfig, HeaderFeature } from '../../header.types';
import { HeaderService } from '../../header.service';

@Component({
    selector: 'app-header-right',
    imports: [
        MatIcon, MatButton, MatMenu, MatMenuItem, MatMenuTrigger,
        MatDivider, MatTooltip, NgTemplateOutlet
    ],
    templateUrl: './header-right.component.html',
    styleUrl: './header-right.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class HeaderRight {
    public readonly config = input<HeaderConfig>();
    protected readonly headerService = inject(HeaderService);
    protected readonly musicState = inject(MusicTracksViewState);
    protected readonly Feature = HeaderFeature;
    protected readonly SortOption = MusicSortOption;
    protected readonly sortOptions = [
        {label: 'SoundCloud Order', value: MusicSortOption.POSITION_ASC, icon: 'reorder'},
        {label: 'Newest First', value: MusicSortOption.DATE_DESC, icon: 'clock_arrow_down'},
        {label: 'Oldest First', value: MusicSortOption.DATE_ASC, icon: 'clock_arrow_up'},
        {label: 'Alphabetical (A-Z)', value: MusicSortOption.NAME_ASC, icon: 'sort_by_alpha'},
        {label: 'Alphabetical (Z-A)', value: MusicSortOption.NAME_DESC, icon: 'sort_by_alpha'},
    ];

    private readonly _dialog = inject(MatDialog);
    private readonly _settingsService = inject(SettingsService);
    private readonly _trackService = inject(MusicTracksViewService);
    private readonly _snackBar = inject(MatSnackBar);
    private readonly _downloadTracking = inject(DownloadTrackingService);
    private readonly _localSyncing = signal(false);

    protected readonly isSyncing = computed(() => this._localSyncing() || this._downloadTracking.activeDownloads().length > 0);

    protected hasFeature(feature: HeaderFeature): boolean {
        return this.config()?.features.includes(feature) ?? false;
    }

    protected syncAll() {
        this._localSyncing.set(true);
        this._settingsService.getSettings().subscribe({
            next: (settings) => {
                if (!settings.soundcloud.profileUrl) {
                    this._snackBar.open('Please configure SoundCloud URL in settings first', 'OK');
                    this._localSyncing.set(false);
                    return;
                }

                this._trackService.download(settings.soundcloud.profileUrl).subscribe({
                    next: () => {
                        setTimeout(() => this._localSyncing.set(false), 2000);
                        this._snackBar.openFromComponent(DownloadProgressSnackbar, {
                            duration: 5000,
                            panelClass: 'download-snackbar',
                            horizontalPosition: 'right',
                            verticalPosition: 'bottom',
                        });
                    },
                    error: () => this._localSyncing.set(false)
                });
            },
            error: () => this._localSyncing.set(false)
        });
    }

    protected openDownloadDialog() {
        this._dialog.open(DownloadDialogComponent, {
            maxWidth: '450px',
            width: '100%',
            autoFocus: 'input',
        });
    }

    protected setSort(option: MusicSortOption) {
        this.musicState.setSortOption(option);
    }
}
