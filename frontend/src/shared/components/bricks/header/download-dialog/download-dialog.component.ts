import {ChangeDetectionStrategy, Component, inject, OnInit, signal} from '@angular/core';
import {MusicTracksViewService} from "@app/pages/music-tracks-view/music-tracks-view.service";
import {DialogRef} from "@angular/cdk/dialog";
import {MatSnackBar} from "@angular/material/snack-bar";
import {form, FormField, required} from "@angular/forms/signals";
import {soundCloudUrl} from "@shared/validators/form.validators";
import {DownloadProgressSnackbar} from "@shared/components/download-progress-snackbar/download-progress-snackbar";
import {ensureStringArray} from "@shared/utils/array.utils";
import {MatError, MatFormField, MatHint, MatInput, MatLabel, MatPrefix} from "@angular/material/input";
import {MatIcon} from "@angular/material/icon";
import {MatDialogActions, MatDialogClose, MatDialogContent, MatDialogTitle} from "@angular/material/dialog";
import {MatList, MatListItem} from "@angular/material/list";
import {MatButton, MatIconButton} from "@angular/material/button";

const STORAGE_KEY = 'vortexdl_download_history';

@Component({
    selector: 'app-download-dialog',
    imports: [
        MatError,
        MatIcon,
        MatPrefix,
        MatHint,
        MatDialogContent,
        MatDialogTitle,
        MatFormField,
        MatLabel,
        FormField,
        MatInput,
        MatList,
        MatListItem,
        MatIconButton,
        MatDialogActions,
        MatButton,
        MatDialogClose
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
    protected readonly urlField = form(this._urlValue, (f) => {
        required(f, {message: 'URL is required'});
        soundCloudUrl(f);
    });

    public ngOnInit() {
        this.loadHistory();
    }

    protected removeFromHistory(item: string) {
        this.history = this.history.filter(i => i !== item);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.history));
    }

    protected selectHistory(item: string) {
        this.urlField().reset(item);
    }

    protected onDownload() {
        if (this.urlField().invalid()) return;
        const url = this.urlField().value();

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
