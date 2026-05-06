import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { DownloadItem, DownloadStatus } from "@app/services/download-tracking.service";
import { MatIcon } from "@angular/material/icon";
import { MatProgressBar } from "@angular/material/progress-bar";
import { NgOptimizedImage } from "@angular/common";

@Component({
  selector: 'app-active-download-item',
  imports: [
    MatIcon,
    MatProgressBar,
    NgOptimizedImage
  ],
  templateUrl: './active-download-item.component.html',
  styleUrl: './active-download-item.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ActiveDownloadItemComponent {
  public readonly item = input.required<DownloadItem>();
  public readonly remove = output<number>();

  protected readonly DownloadStatus = DownloadStatus;

  public onRemove() {
    this.remove.emit(this.item().id);
  }
}
