import {Component} from '@angular/core';
import {RouterOutlet} from '@angular/router';
import {Sidebar} from '@shared/components/sidebar/sidebar';
import {GlobalLoader} from '@shared/components/global-loader/global-loader';
import {PlayerComponent} from '@shared/components/player/player';

@Component({
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.scss',
    imports: [RouterOutlet, Sidebar, GlobalLoader, PlayerComponent],
})
export class App {
}
