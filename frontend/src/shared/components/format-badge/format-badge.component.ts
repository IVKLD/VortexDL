import { Component, input } from '@angular/core';
import { AudioFormat } from '../../models/music-track.model';

@Component({
  selector: 'app-format-badge',
  template: '{{ format() }}',
  styleUrl: './format-badge.component.scss',
  host: {
    '[attr.data-format]': 'format().toLowerCase()',
  },
})
export class FormatBadgeComponent {
  public readonly format = input.required<AudioFormat | string>();
}
