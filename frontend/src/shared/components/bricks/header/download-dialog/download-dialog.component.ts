import {ChangeDetectionStrategy, Component, inject, OnInit, signal} from '@angular/core';
import {MatButton, MatIconButton} from '@angular/material/button';
import {MatDialogActions, MatDialogClose, MatDialogContent, MatDialogTitle} from '@angular/material/dialog';
import {MatFormField, MatHint, MatLabel, MatPrefix} from '@angular/material/form-field';
import {MatInput} from '@angular/material/input';
import {MatList, MatListItem} from '@angular/material/list';
import {MusicTracksViewService} from '@app/pages/music-tracks-view/music-tracks-view.service';
import {DialogRef} from '@angular/cdk/dialog';
import {DownloadProgressSnackbar} from '@shared/components/download-progress-snackbar/download-progress-snackbar';
import {MatSnackBar} from '@angular/material/snack-bar';
import {MatIcon} from '@angular/material/icon';
import {form, FormField, required} from '@angular/forms/signals';
import {ensureStringArray} from '@shared/utils/array.utils';

const STORAGE_KEY = 'vortexdl_download_history';

@Component({
    selector: 'app-download-dialog',
    imports: [
        FormField,
        MatDialogTitle,
        MatDialogContent,
        MatFormField,
        MatLabel,
        MatHint,
        MatList,
        MatListItem,
        MatDialogClose,
        MatDialogActions,
        MatButton,
        MatInput,
        MatIconButton,
        MatIcon,
        MatPrefix,
    ],
    templateUrl: './download-dialog.component.html',
    styleUrl: './download-dialog.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DownloadDialogComponent implements OnInit {
    private readonly _trackService = inject(MusicTracksViewService);
    private readonly _dialogRef = inject(DialogRef);
    private readonly _snackBar = inject(MatSnackBar);
    private readonly _urlValue = signal('');

    protected history: string[] = [];
    protected readonly urlForm = form(this._urlValue, (f) => {
        required(f, {message: 'URL is required'});
    });

    public ngOnInit() {
        this.loadHistory();
    }

    protected removeFromHistory(item: string) {
        this.history = this.history.filter(i => i !== item);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.history));
    }

    protected selectHistory(item: string) {
        this.urlForm().reset(item);
    }

    protected onDownload() {
        if (this.urlForm().invalid()) return;
        const url = this.urlForm().value();

        this.addToHistory(url);
        this._trackService.download(url).subscribe({
            next: () => {
                this._dialogRef.close();

                this._snackBar.openFromComponent(DownloadProgressSnackbar, {
                    duration: 5000,
                    panelClass: 'download-snackbar',
                    horizontalPosition: 'right',
                    verticalPosition: 'bottom',
                });
            },
            error: err => {
                console.error('Download failed', err);
            },
        });
    }

    private loadHistory() {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (!saved) return;

        try {
            const parsed = JSON.parse(saved);
            this.history = ensureStringArray(parsed);
        } catch {
            this.history = [];
        }
    }

    private addToHistory(url: string) {
        if (!this.history.includes(url)) {
            this.history = [url, ...this.history.slice(0, 4)];
            localStorage.setItem(STORAGE_KEY, JSON.stringify(this.history));
        }
    }
}
