import {RouterOutlet} from '@angular/router';
import {Sidebar} from '@shared/components/sidebar/sidebar';
import {GlobalLoader} from '@shared/components/global-loader/global-loader';
import {ChangeDetectionStrategy, Component} from "@angular/core";

@Component({
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.scss',
    imports: [RouterOutlet, Sidebar, GlobalLoader],
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
}
