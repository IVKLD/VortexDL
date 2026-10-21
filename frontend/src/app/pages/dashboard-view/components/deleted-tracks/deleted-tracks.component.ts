import { Component, input, output } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { DatePipe } from '@angular/common';
import { FileSizePipe } from '@shared/pipes/file-size.pipe';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { DeletedTrack } from '../../dashboard-view.model';

@Component({
    selector: 'app-deleted-tracks',
    imports: [MatIcon, MatButton, DatePipe, FileSizePipe, EmptyPaneComponent],
    templateUrl: './deleted-tracks.component.html',
    styleUrl: './deleted-tracks.component.scss',
})
export class DeletedTracksComponent {
    public readonly recentDeleted = input.required<DeletedTrack[]>();
    public readonly totalDeleted = input.required<number>();
    public readonly totalDeletedBytes = input.required<number>();

    public readonly clearHistory = output<void>();

    protected onClearHistory(): void {
        this.clearHistory.emit();
    }
}
