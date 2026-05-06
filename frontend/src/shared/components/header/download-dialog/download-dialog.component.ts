import {ChangeDetectionStrategy, Component, inject, OnInit} from '@angular/core';
import {FormControl, ReactiveFormsModule, Validators} from '@angular/forms';
import {MatButton, MatIconButton} from '@angular/material/button';
import {MatDialogActions, MatDialogClose, MatDialogContent, MatDialogTitle} from '@angular/material/dialog';
import {MatFormField, MatHint, MatLabel} from '@angular/material/form-field';
import {MatInput} from '@angular/material/input';
import {MatList, MatListItem} from '@angular/material/list';
import {MusicTracksViewService} from "@app/pages/music-tracks-view/music-tracks-view.service";
import {DialogRef} from "@angular/cdk/dialog";
import {urlValidator} from "@shared/validators/url.validator";
import {DownloadProgressSnackbar} from "@shared/components/download-progress-snackbar/download-progress-snackbar";
import {MatSnackBar} from "@angular/material/snack-bar";
import {MatIcon} from "@angular/material/icon";

const STORAGE_KEY = 'vortexdl_download_history';

@Component({
    selector: 'app-download-dialog',
    imports: [
        ReactiveFormsModule,
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
        MatIcon
    ],
    templateUrl: './download-dialog.component.html',
    styleUrl: './download-dialog.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DownloadDialogComponent implements OnInit {
    private readonly _trackService = inject(MusicTracksViewService);
    private readonly _dialogRef = inject(DialogRef);
    private readonly _snackBar = inject(MatSnackBar);

    public url = new FormControl<string>('', [Validators.required, urlValidator()]);
    public history: string[] = [];

    ngOnInit() {
        this.loadHistory();
    }

    private loadHistory() {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
            try {
                this.history = JSON.parse(saved) as string[];
            } catch (e) {
                this.history = [];
            }
        }
    }

    private addToHistory(url: string) {
        if (!this.history.includes(url)) {
            this.history = [url, ...this.history.slice(0, 4)];
            localStorage.setItem(STORAGE_KEY, JSON.stringify(this.history));
        }
    }

    protected selectHistory(item: string) {
        this.url.setValue(item);
    }

    removeFromHistory(item: string) {
        this.history = this.history.filter((i) => i !== item)
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.history));
    }

    protected onDownload() {
        const url = this.url.getRawValue()!

        this.addToHistory(url);
        this._trackService.download(url).subscribe({
            next: () => {
                this._dialogRef.close();

                this._snackBar.openFromComponent(DownloadProgressSnackbar, {
                    duration: 5000,
                    panelClass: 'download-snackbar',
                    horizontalPosition: 'right',
                    verticalPosition: 'bottom'
                });
            },
            error: (err: any) => {
                console.error('Download failed', err);
            }
        });
    }
}
