import { booleanAttribute, Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';

export type StatusBadgeVariant = 'success' | 'error' | 'info' | 'checking' | 'idle';

@Component({
  selector: 'app-status-badge',
  imports: [MatIcon],
  templateUrl: './status-badge.component.html',
  styleUrl: './status-badge.component.scss',
  host: {
    '[class]': 'variant()',
  },
})
export class StatusBadgeComponent {
  public readonly variant = input<StatusBadgeVariant>('idle');
  public readonly icon = input<string | null>(null);
  public readonly dot = input(false, { transform: booleanAttribute });
  public readonly rotating = input(false, { transform: booleanAttribute });
}
