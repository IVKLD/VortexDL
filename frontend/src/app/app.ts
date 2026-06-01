import {ChangeDetectionStrategy, Component} from '@angular/core';
import {RouterOutlet} from '@angular/router';
import {Sidebar} from '@shared/components/sidebar/sidebar';
import {GlobalLoader} from '@shared/components/global-loader/global-loader';

@Component({
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.scss',
    imports: [RouterOutlet, Sidebar, GlobalLoader],
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
}
