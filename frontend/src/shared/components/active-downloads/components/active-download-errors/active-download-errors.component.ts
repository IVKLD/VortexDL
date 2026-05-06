import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatIcon } from "@angular/material/icon";

@Component({
  selector: 'app-active-download-errors',
  imports: [
    MatIcon
  ],
  templateUrl: './active-download-errors.component.html',
  styleUrl: './active-download-errors.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ActiveDownloadErrorsComponent {
  public readonly errors = input.required<string[]>();
  public readonly clear = output<void>();

  public onClear() {
    this.clear.emit();
  }
}
