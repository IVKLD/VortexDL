import {Component} from '@angular/core';
import {RouterOutlet} from '@angular/router';
import {Sidebar} from "../shared/components/sidebar/sidebar";
import {Header} from "../shared/components/header/header";

@Component({
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.scss',
    imports: [
        RouterOutlet,
        Sidebar,
        Header,
    ]
})
export class App {}
