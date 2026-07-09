import {Component, inject, OnInit, signal} from '@angular/core';
import {MusicTracksViewService} from "@app/pages/music-tracks-view/music-tracks-view.service";
import {DialogRef} from "@angular/cdk/dialog";
import {form, FormField, required} from "@angular/forms/signals";
import {soundCloudUrl} from "@shared/validators/form.validators";
import {ensureStringArray} from "@shared/utils/array.utils";
import {MatError, MatFormField, MatHint, MatInput, MatLabel} from "@angular/material/input";
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
    })
export class DownloadDialogComponent implements OnInit {
    private readonly _trackService = inject(MusicTracksViewService);
    private readonly _dialogRef = inject(DialogRef);
    private readonly _urlValue = signal('');

    protected readonly history = signal<string[]>([]);
    protected readonly urlField = form(this._urlValue, (f) => {
        required(f, {message: 'URL is required'});
        soundCloudUrl(f);
    });

    private loadHistory() {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (!saved) return;

        try {
            const parsed = JSON.parse(saved);
            this.history.set(ensureStringArray(parsed));
        } catch {
            this.history.set([]);
        }
    }

    private addToHistory(url: string) {
        if (!this.history().includes(url)) {
            const next = [url, ...this.history().slice(0, 4)];
            this.history.set(next);
            localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
        }
    }

    protected removeFromHistory(item: string) {
        const next = this.history().filter(i => i !== item);
        this.history.set(next);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
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
            },
            error: err => {
                console.error('Download failed', err);
            },
        });
    }

    public ngOnInit() {
        this.loadHistory();
    }
}
