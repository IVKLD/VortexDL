import {Component, inject, signal} from '@angular/core';
import {MusicTracksService} from "@app/pages/music-tracks-view/music-tracks.service";
import {DownloadHistoryService} from "@app/services/download-history.service";
import {DialogRef} from "@angular/cdk/dialog";
import {form, FormField, required} from "@angular/forms/signals";
import {downloadUrl} from "@shared/validators/form.validators";
import {MatError, MatFormField, MatHint, MatInput, MatLabel} from "@angular/material/input";
import {MatIcon} from "@angular/material/icon";
import {MatDialogActions, MatDialogClose, MatDialogContent, MatDialogTitle} from "@angular/material/dialog";
import {MatList, MatListItem} from "@angular/material/list";
import {MatButton, MatIconButton} from "@angular/material/button";

import {PlatformChipComponent} from "@shared/components/platform-chip/platform-chip.component";
import {PlatformPipe} from "@shared/pipes/platform.pipe";

@Component({
    selector: 'app-download-dialog',
    imports: [
        MatError,
        MatIcon,
        MatHint,
        MatDialogContent,
        MatFormField,
        MatLabel,
        FormField,
        MatInput,
        MatList,
        MatListItem,
        MatIconButton,
        MatDialogActions,
        MatButton,
        MatDialogClose,
        MatDialogTitle,
        PlatformChipComponent,
        PlatformPipe,
    ],
    templateUrl: './download-dialog.component.html',
    styleUrl: './download-dialog.component.scss',
})
export class DownloadDialogComponent {
    private readonly _trackService = inject(MusicTracksService);
    private readonly _historyService = inject(DownloadHistoryService);
    private readonly _dialogRef = inject(DialogRef);
    private readonly _urlValue = signal('');

    protected readonly history = this._historyService.history;
    protected readonly urlField = form(this._urlValue, (f) => {
        required(f, {message: 'URL is required'});
        downloadUrl(f);
    });

    protected removeFromHistory(item: string) {
        this._historyService.remove(item);
    }

    protected selectHistory(item: string) {
        this.urlField().reset(item);
    }

    protected onDownload() {
        if (this.urlField().invalid()) return;
        const url = this.urlField().value();

        this._historyService.add(url);
        this._trackService.download(url).subscribe({
            next: () => {
                this._dialogRef.close();
            },
            error: err => {
                console.error('Download failed', err);
            },
        });
    }
}
