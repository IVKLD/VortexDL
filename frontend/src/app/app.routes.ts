import {DashboardView} from "./pages/dashboard-view/dashboard-view";
import {PlaylistView} from "./pages/playlist-view/playlist-view";
import {MusicTracksView} from "./pages/music-tracks-view/music-tracks-view";
import {Route} from "@angular/router";

export type RouteData = {
    text: string;
}

export type CustomRoute = {
    readonly data: RouteData;
} & Route

export type CustomRoutes = CustomRoute[]

export const routes: CustomRoutes = [
    {
        path: '',
        title: 'Dashboard',
        data:
            {
                text: 'dashboard',
            },
        component: DashboardView
    },
    {
        path: 'playlist',
        title: 'Playlist',
        data:
            {
                text: 'playlist',
            },
        component: PlaylistView
    },
    {
        path: 'musics',
        title: 'Musics',
        data:
            {
                text: 'musics',
            },
        component: MusicTracksView
    }
];
