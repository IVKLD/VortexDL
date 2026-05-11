import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { Sidebar } from '@shared/components/sidebar/sidebar';

@Component({
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.scss',
    standalone: true,
    imports: [RouterOutlet, Sidebar],
})
export class App { }
