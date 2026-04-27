import {Component} from '@angular/core';
import {PlaylistViewService} from "./playlist-view.service";

@Component({
    selector: 'app-playlist-view',
    providers: [PlaylistViewService],
    templateUrl: './playlist-view.html',
    styleUrl: './playlist-view.scss'
})
export class PlaylistView {

}
