import {DashboardView} from './pages/dashboard-view/dashboard-view';
import {MusicTracksView} from './pages/music-tracks-view/music-tracks-view';
import {Route} from '@angular/router';
import {HeaderConfig, HeaderFeature} from '@shared/components/bricks/header/header.types';
import {SettingsView} from './pages/settings-view/settings-view.component';

export interface RouteData {
    header?: HeaderConfig;
}

export type CustomRoute = {
    readonly data: RouteData;
} & Route;

export type CustomRoutes = CustomRoute[];

export const routes: CustomRoutes = [
    {
        path: '',
        title: 'Dashboard',
        data: {
            header: {
                title: 'Overview',
                features: [HeaderFeature.Stats],
            },
        },
        component: DashboardView,
    },
    {
        path: 'musics',
        title: 'Musics',
        data: {
            header: {
                title: 'Library',
                features: [HeaderFeature.Stats, HeaderFeature.Search, HeaderFeature.Sort],
            },
        },
        component: MusicTracksView,
    },
    {
        path: 'settings',
        title: 'Settings',
        data: {
            header: {
                title: 'Control Panel',
                features: [HeaderFeature.Stats],
            },
        },
        component: SettingsView,
    },
];

