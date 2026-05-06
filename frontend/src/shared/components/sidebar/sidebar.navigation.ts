export interface SidebarNavItem {
    path: string;
    icon: string;
    label: string;
    exact?: boolean;
}

export const SIDEBAR_NAV_ITEMS: SidebarNavItem[] = [
    {
        path: '/',
        icon: 'dashboard',
        label: 'Dashboard',
        exact: true
    },
    {
        path: '/playlist',
        icon: 'cards_stack',
        label: 'Playlist'
    },
    {
        path: '/musics',
        icon: 'library_music',
        label: 'Library'
    },
    {
        path: '/settings',
        icon: 'settings',
        label: 'Settings'
    }
];
