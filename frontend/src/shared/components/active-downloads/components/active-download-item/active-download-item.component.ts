import {Component, input, output} from '@angular/core';
import {DownloadItem, DownloadStatus} from '@app/services/download-tracking.service';
import {MatIcon} from '@angular/material/icon';
import {MatProgressBar} from '@angular/material/progress-bar';
import {NgOptimizedImage} from '@angular/common';
import {MatIconButton} from "@angular/material/button";

@Component({
    selector: 'app-active-download-item',
    imports: [MatIcon, MatProgressBar, NgOptimizedImage, MatIconButton],
    templateUrl: './active-download-item.component.html',
    styleUrl: './active-download-item.component.scss',
})
export class ActiveDownloadItemComponent {
    public readonly item = input.required<DownloadItem>();
    protected readonly DownloadStatus = DownloadStatus;
    public readonly remove = output<number>();

    protected get hasProgress(): boolean {
        return (this.item().progress ?? 0) > 0;
    }

    protected get progressPercent(): number {
        return Math.round(this.item().progress ?? 0);
    }

    protected onRemove() {
        this.remove.emit(this.item().id);
    }
}

