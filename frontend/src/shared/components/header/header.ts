import {ChangeDetectionStrategy, Component, computed, inject, signal} from '@angular/core';
import {MatButton} from '@angular/material/button';
import {MatDialog} from '@angular/material/dialog';
import {DownloadDialogComponent} from './download-dialog/download-dialog.component';
import {MatIcon} from '@angular/material/icon';
import {MatMenu, MatMenuItem, MatMenuTrigger} from '@angular/material/menu';
import {MusicSortOption, MusicTracksViewState} from '@app/pages/music-tracks-view/music-tracks-view.state';
import {ActivatedRoute, NavigationEnd, Router} from '@angular/router';
import {MatFormField, MatPrefix} from '@angular/material/form-field';
import {MatInput} from '@angular/material/input';
import {HeaderConfig, HeaderFeature} from './header.types';
import {distinctUntilChanged, filter, map} from 'rxjs';
import {takeUntilDestroyed, toObservable, toSignal} from '@angular/core/rxjs-interop';
import {debounce, form, FormField} from '@angular/forms/signals';
import {RouteData} from '@app/app.routes';
import {MatDivider} from "@angular/material/list";
import {SettingsService} from '@app/pages/settings/settings.service';
import {MusicTracksViewService} from '@app/pages/music-tracks-view/music-tracks-view.service';
import {MatSnackBar} from '@angular/material/snack-bar';
import {DownloadProgressSnackbar} from '@shared/components/download-progress-snackbar/download-progress-snackbar';
import {DownloadTrackingService} from '@app/services/download-tracking.service';

@Component({
    selector: 'app-header',
    imports: [MatIcon, MatButton, MatMenu, MatMenuItem, MatMenuTrigger, MatFormField, MatInput, MatPrefix, MatDivider, FormField],
    templateUrl: './header.html',
    styleUrl: './header.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Header {
    protected readonly musicState = inject(MusicTracksViewState);
    protected readonly Feature = HeaderFeature;
    protected readonly SortOption = MusicSortOption;
    protected readonly sortOptions = [
        {label: 'Alphabetical (A-Z)', value: MusicSortOption.NAME_ASC, icon: 'sort_by_alpha'},
        {label: 'Alphabetical (Z-A)', value: MusicSortOption.NAME_DESC, icon: 'sort_by_alpha'},
        {label: 'Date Added', value: MusicSortOption.DATE, icon: 'schedule'},
    ];
    private readonly _dialog = inject(MatDialog);
    private readonly _router = inject(Router);
    private readonly _route = inject(ActivatedRoute);
    private readonly _trackService = inject(MusicTracksViewService);
    private readonly _settingsService = inject(SettingsService);
    private readonly _snackBar = inject(MatSnackBar);
    private readonly _downloadTracking = inject(DownloadTrackingService);
    private readonly _localSyncing = signal(false);
    protected readonly isSyncing = computed(() => this._localSyncing() || this._downloadTracking.activeDownloads().length > 0);

    private readonly _searchValue = signal(this.musicState.searchQuery());
    protected readonly searchForm = form(this._searchValue, (p) => {
        debounce(p, 200);
    });
    private readonly _headerConfig$ = this._router.events.pipe(
        filter(event => event instanceof NavigationEnd),
        map(() => {
            let route = this._route.root;
            while (route.firstChild) {
                route = route.firstChild;
            }
            const data: RouteData = route.snapshot.data;
            return data.header;
        }),
    );
    protected readonly config = toSignal(this._headerConfig$, {
        initialValue: this.getInitialHeaderConfig(),
    });

    constructor() {
        toObservable(this.searchForm().value).pipe(
            distinctUntilChanged(),
            takeUntilDestroyed()
        ).subscribe(query => this.musicState.setSearchQuery(query));
    }

    protected hasFeature(feature: HeaderFeature): boolean {
        return this.config()?.features.includes(feature) ?? false;
    }

    protected syncAll() {
        this._localSyncing.set(true);
        this._settingsService.getSettings().subscribe({
            next: (settings) => {
                if (!settings.soundcloud.profileUrl) {
                    this._snackBar.open('Please configure SoundCloud URL in settings first', 'OK', {duration: 5000});
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

    private getInitialHeaderConfig(): HeaderConfig | undefined {
        const data: RouteData = this._route.root.snapshot.firstChild?.data || {};
        return data.header;
    }
}
