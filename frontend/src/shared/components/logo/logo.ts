import { ChangeDetectionStrategy, Component } from '@angular/core';

@Component({
  selector: 'app-logo',
  templateUrl: './logo.html',
  styleUrls: ['./logo.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LogoComponent { }
