import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { Track } from '../music-tracks-view.service';
import { MatButton, MatIconButton } from '@angular/material/button';
import { MatIcon } from '@angular/material/icon';

@Component({
  selector: 'app-music-detail-modal',
  imports: [
    MatButton,
    MatIconButton,
    MatIcon
  ],
  templateUrl: './music-detail-modal.html',
  styleUrl: './music-detail-modal.scss',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class MusicDetailModal {
  public readonly track: Track = inject(MAT_DIALOG_DATA);
  private readonly dialogRef = inject(MatDialogRef<MusicDetailModal>);

  protected displayTitle(): string {
    return this.track.filename.replace(`.${this.track.format}`, '').replace(/_/g, ' ');
  }

  protected close(): void {
    this.dialogRef.close();
  }
}
