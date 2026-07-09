import {RouterOutlet} from '@angular/router';
import {Sidebar} from '@shared/components/bricks/sidebar/sidebar';
import {Component} from '@angular/core';

@Component({
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.scss',
    imports: [RouterOutlet, Sidebar],
    })
export class App {
}
