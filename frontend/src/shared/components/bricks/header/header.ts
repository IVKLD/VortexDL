import { Component, computed, inject, signal } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router } from '@angular/router';
import { HeaderFeature } from './header.types';
import { filter, finalize } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { HeaderService } from './header.service';

import { MatFormField, MatPrefix } from '@angular/material/form-field';
import { MatInput } from '@angular/material/input';
import { MatIcon } from '@angular/material/icon';
import { FormField } from '@angular/forms/signals';
import { MatButton } from '@angular/material/button';
import { MatMenu, MatMenuItem, MatMenuTrigger } from '@angular/material/menu';
import { MatTooltip } from '@angular/material/tooltip';
import { MatDivider } from '@angular/material/divider';

import { MatDialog } from '@angular/material/dialog';
import { SettingsService } from '@app/pages/settings-view/settings.service';
import { MusicTracksViewService } from '@app/pages/music-tracks-view/music-tracks-view.service';
import { NotificationService } from '@app/services/notification.service';
import { DownloadTrackingService } from '@app/services/download-tracking.service';
import { DownloadDialogComponent } from './download-dialog/download-dialog.component';

@Component({
    selector: 'app-header',
    imports: [
        MatFormField, MatInput, MatPrefix, MatIcon, FormField,
        MatButton, MatMenu, MatMenuItem, MatMenuTrigger, MatDivider, MatTooltip
    ],
    templateUrl: './header.html',
    styleUrl: './header.scss',
})
export class Header {
    private readonly _router = inject(Router);
    private readonly _route = inject(ActivatedRoute);

    private readonly _musicState = inject(MusicTracksViewState);

    private readonly _dialog = inject(MatDialog);
    private readonly _settingsService = inject(SettingsService);
    private readonly _trackService = inject(MusicTracksViewService);
    private readonly _notification = inject(NotificationService);
    private readonly _downloadTracking = inject(DownloadTrackingService);
    protected readonly headerService = inject(HeaderService);
    private readonly _localSyncing = signal(false);
    protected readonly Feature = HeaderFeature;
    protected readonly tracksCount = computed(() => this._musicState.tracks().length);

    protected readonly isSyncing = computed(() =>
        this._localSyncing() ||
        this._downloadTracking.syncingUrls().length > 0 ||
        this._downloadTracking.activeDownloads().length > 0
    );

    protected readonly activeSortLabel = computed(() => {
        const bind = this.headerService.sortBind();
        if (!bind) return '';
        const val = bind.value();
        const opt = this.headerService.config()?.sortOptions?.find(o => o.value === val);
        return opt ? (opt.shortLabel || opt.label) : '';
    });

    constructor() {
        this.updateHeader();

        this._router.events.pipe(
            filter(event => event instanceof NavigationEnd),
            takeUntilDestroyed()
        ).subscribe(() => this.updateHeader());
    }

    private updateHeader() {
        let route = this._route.root;
        while (route.firstChild) {
            route = route.firstChild;
        }
        const data = route.snapshot.data;
        this.headerService.config.set(data['header']);
        this.headerService.searchBind.set(data['headerSearch'] || null);
        this.headerService.sortBind.set(data['headerSort'] || null);
    }

    protected onSearchKeydown(event: KeyboardEvent): void {
        const bind = this.headerService.searchBind();
        if (event.key === 'Enter' && bind && bind.onSubmit) {
            bind.onSubmit(bind.formField().value().trim());
        }
    }

    protected syncAll() {
        this._localSyncing.set(true);
        this._settingsService.getSettings().subscribe({
            next: (settings) => {
                if (!settings.soundcloud.profileUrl) {
                    this._notification.error('Please configure SoundCloud URL in settings first');
                    this._localSyncing.set(false);
                    return;
                }

                this._trackService.download(settings.soundcloud.profileUrl)
                    .pipe(finalize(() => this._localSyncing.set(false)))
                    .subscribe();
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
}


