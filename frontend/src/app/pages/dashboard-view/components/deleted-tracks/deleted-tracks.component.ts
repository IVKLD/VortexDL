import { Component, input, output } from '@angular/core';
import {
    MatList,
    MatListItem,
    MatListItemIcon,
    MatListItemLine,
    MatListItemMeta,
    MatListItemTitle,
} from '@angular/material/list';
import { MatIcon } from '@angular/material/icon';
import { MatButton } from '@angular/material/button';
import { DatePipe } from '@angular/common';
import { FileSizePipe } from '@shared/pipes/file-size.pipe';
import { EmptyPaneComponent } from '@shared/components/empty-pane/empty-pane';
import { FormatBadgeComponent } from '@shared/components/format-badge/format-badge.component';
import { DeletedTrack } from './deleted-tracks.model';

@Component({
    selector: 'app-deleted-tracks',
    imports: [
        MatList,
        MatListItem,
        MatListItemIcon,
        MatListItemTitle,
        MatListItemLine,
        MatListItemMeta,
        MatIcon,
        MatButton,
        DatePipe,
        FileSizePipe,
        EmptyPaneComponent,
        FormatBadgeComponent,
    ],
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
