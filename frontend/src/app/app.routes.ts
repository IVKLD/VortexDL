import { DashboardView } from "./pages/dashboard-view/dashboard-view";
import { PlaylistView } from "./pages/playlist-view/playlist-view";
import { MusicTracksView } from "./pages/music-tracks-view/music-tracks-view";
import { Route } from "@angular/router";
import { HeaderFeature, HeaderConfig } from "@shared/components/header/header.types";

export interface RouteData {
    header?: HeaderConfig;
}

export type CustomRoute = {
    readonly data: RouteData;
} & Route

export type CustomRoutes = CustomRoute[]

export const routes: CustomRoutes = [
    {
        path: '',
        title: 'Dashboard',
        data: {
            header: {
                title: 'Overview',
                features: [HeaderFeature.Stats]
            }
        },
        component: DashboardView
    },
    {
        path: 'playlist',
        title: 'Playlist',
        data: {
            header: {
                title: 'Playlists',
                features: [HeaderFeature.Search]
            }
        },
        component: PlaylistView
    },
    {
        path: 'musics',
        title: 'Musics',
        data: {
            header: {
                title: 'Library',
                features: [HeaderFeature.Search, HeaderFeature.Sort, HeaderFeature.Stats, HeaderFeature.Download]
            }
        },
        component: MusicTracksView
    },
    {
        path: 'settings',
        title: 'Settings',
        data: {
            header: {
                title: 'Control Panel',
                features: []
            }
        },
        component: MusicTracksView
    }
];
